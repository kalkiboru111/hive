//! Bot engine — wraps whatsapp-rust, routes incoming messages through handlers.
//!
//! The engine:
//! 1. Connects to WhatsApp via whatsapp-rust's Bot builder
//! 2. Listens for incoming messages via the Event system
//! 3. Looks up conversation state for each sender
//! 4. Routes through the handler chain
//! 5. Sends responses and persists state

pub mod conversation;

use crate::config::HiveConfig;
use crate::handlers::{self, HandlerResult, MessageContext};
use crate::network::service::{NetworkNotifier, NetworkService};
use crate::payments::{MpesaClient, PaymentProvider};
use crate::store::Store;
use anyhow::Result;
use conversation::ConversationState;
use log::{error, info, warn};
use std::path::PathBuf;
use std::sync::Arc;
use whatsapp_rust::bot::{Bot, MessageContext as WaMessageContext};
use whatsapp_rust::pair_code::PairCodeOptions;
use whatsapp_rust::types::events::Event;
use whatsapp_rust_sqlite_storage::SqliteStore as WaSqliteStore;
use whatsapp_rust_tokio_transport::TokioWebSocketTransportFactory;
use whatsapp_rust_ureq_http_client::UreqHttpClient;

/// Core bot engine that ties everything together.
pub struct BotEngine {
    config: Arc<HiveConfig>,
    store: Store,
    project_dir: PathBuf,
    phone_number: Option<String>,
    network_notifier: NetworkNotifier,
    payment_provider: Option<Arc<dyn PaymentProvider>>,
    wa_client_shared: Option<Arc<tokio::sync::RwLock<Option<Arc<whatsapp_rust::client::Client>>>>>,
}

impl BotEngine {
    /// Create a new bot engine.
    pub async fn new(config: HiveConfig, store: Store, project_dir: PathBuf) -> Result<Self> {
        // Initialize Reality Network integration if enabled
        let network_notifier = if config.network.enabled {
            let (service, notifier) = NetworkService::new(
                &config.network,
                store.clone(),
                config.business.name.clone(),
                &project_dir,
            )
            .await?;

            // Spawn the network service as a background task
            tokio::spawn(async move {
                service.run().await;
            });

            notifier
        } else {
            info!("🌐 Reality Network integration disabled (set network.enabled: true to enable)");
            NetworkNotifier::disabled()
        };

        // Initialize payment provider if configured
        let payment_provider: Option<Arc<dyn PaymentProvider>> = if config.payments.enabled {
            if let Some(ref mpesa_cfg) = config.payments.mpesa {
                info!("💰 M-Pesa payments enabled ({})", 
                      if mpesa_cfg.sandbox { "sandbox" } else { "production" });
                let mpesa_config = crate::payments::mpesa::MpesaConfig {
                    consumer_key: mpesa_cfg.consumer_key.clone(),
                    consumer_secret: mpesa_cfg.consumer_secret.clone(),
                    shortcode: mpesa_cfg.shortcode.clone(),
                    passkey: mpesa_cfg.passkey.clone(),
                    callback_url: mpesa_cfg.callback_url.clone(),
                    sandbox: mpesa_cfg.sandbox,
                };
                Some(Arc::new(MpesaClient::new(mpesa_config)))
            } else {
                warn!("💰 Payments enabled but no provider configured");
                None
            }
        } else {
            None
        };

        Ok(Self {
            config: Arc::new(config),
            store,
            project_dir,
            phone_number: None,
            network_notifier,
            payment_provider,
            wa_client_shared: None,
        })
    }

    /// Set a phone number for pair code authentication (alternative to QR scanning).
    pub fn with_phone_number(mut self, phone: String) -> Self {
        self.phone_number = Some(phone);
        self
    }

    /// Set shared WhatsApp client for dashboard access.
    pub fn with_wa_client_shared(
        mut self,
        shared: Arc<tokio::sync::RwLock<Option<Arc<whatsapp_rust::client::Client>>>>,
    ) -> Self {
        self.wa_client_shared = Some(shared);
        self
    }

    /// Start the bot — connects to WhatsApp and begins processing messages.
    pub async fn run(&mut self) -> Result<()> {
        info!("Initializing WhatsApp connection...");

        // Set up the whatsapp-rust storage backend
        let wa_db_path = self
            .project_dir
            .join("data")
            .join("whatsapp.db")
            .to_string_lossy()
            .to_string();

        let backend = Arc::new(WaSqliteStore::new(&wa_db_path).await?)
            as Arc<dyn whatsapp_rust::store::traits::Backend>;

        // Build shared state for the event handler closure
        let config = self.config.clone();
        let store = self.store.clone();
        let network_notifier = self.network_notifier.clone();
        let payment_provider = self.payment_provider.clone();
        let wa_client_shared = self.wa_client_shared.clone();

        let mut builder = Bot::builder()
            .with_backend(backend)
            .with_transport_factory(TokioWebSocketTransportFactory::new())
            .with_http_client(UreqHttpClient::new());

        // If phone number is provided, use pair code authentication instead of QR
        if let Some(ref phone) = self.phone_number {
            info!("📱 Using pair code authentication for {}", phone);
            builder = builder.with_pair_code(PairCodeOptions {
                phone_number: phone.clone(),
                ..Default::default()
            });
        }

        let mut bot = builder
            .on_event(move |event, client| {
                let config = config.clone();
                let store = store.clone();
                let network_notifier = network_notifier.clone();
                let payment_provider = payment_provider.clone();
                let wa_client_shared = wa_client_shared.clone();
                async move {
                    match event {
                        Event::PairingQrCode { code, timeout } => {
                            println!("\n📱 Scan this QR code with WhatsApp:");
                            // Generate QR code for terminal display
                            if let Ok(qr) = qrcode::QrCode::new(&code) {
                                let string = qr
                                    .render::<char>()
                                    .quiet_zone(true)
                                    .module_dimensions(2, 1)
                                    .build();
                                println!("{}", string);

                                // Also save as PNG for remote scanning
                                let img = qr.render::<image::Luma<u8>>()
                                    .quiet_zone(true)
                                    .min_dimensions(600, 600)
                                    .build();
                                let png_path = "/tmp/hive-qr.png";
                                if let Err(e) = img.save(png_path) {
                                    warn!("Failed to save QR PNG: {}", e);
                                } else {
                                    info!("📸 QR code saved to {}", png_path);
                                }
                            } else {
                                println!("QR Data: {}", code);
                            }
                            println!(
                                "⏱  Code expires in {} seconds\n",
                                timeout.as_secs()
                            );
                        }
                        Event::PairingCode { code, timeout } => {
                            println!(
                                "\n🔑 Enter this pairing code on your phone: {}",
                                code
                            );
                            println!(
                                "⏱  Code expires in {} seconds\n",
                                timeout.as_secs()
                            );
                        }
                        Event::Connected(_) => {
                            info!("✅ Connected to WhatsApp!");
                            
                            // Populate shared client for dashboard webhook access
                            if let Some(ref shared) = wa_client_shared {
                                let mut client_lock = shared.write().await;
                                *client_lock = Some(client.clone());
                                info!("📡 WhatsApp client shared with dashboard");
                            }
                        }
                        Event::Disconnected(_) => {
                            warn!("⚠️  Disconnected from WhatsApp");
                        }
                        Event::LoggedOut(logout) => {
                            error!(
                                "🚫 Logged out from WhatsApp: {:?}",
                                logout.reason
                            );
                        }
                        Event::Message(message, info) => {
                            // Build our context from the whatsapp-rust event
                            let wa_ctx = WaMessageContext {
                                message,
                                info: info.clone(),
                                client: client.clone(),
                            };

                            match handle_incoming_message(&config, &store, &wa_ctx, &payment_provider).await {
                                Ok(state_changed) => {
                                    if state_changed {
                                        network_notifier.mark_dirty();
                                    }
                                }
                                Err(e) => {
                                    error!(
                                        "Error handling message from {}: {}",
                                        info.source.sender, e
                                    );
                                }
                            }
                        }
                        _ => {
                            // Ignore other events (receipts, presence, etc.)
                        }
                    }
                }
            })
            .build()
            .await?;

        info!("🐝 Bot is starting — waiting for WhatsApp connection...");

        // Run the bot (blocks until disconnected)
        let handle = bot.run().await?;
        handle.await.map_err(|e| anyhow::anyhow!("Bot task panicked: {}", e))?;

        Ok(())
    }
}

/// Handle a single incoming WhatsApp message.
///
/// Returns Ok(true) if store state changed (order/voucher), triggering
/// a Reality Network snapshot submission.
///
/// This is the core routing logic:
/// 1. Extract text from the message
/// 2. Load conversation state for this sender
/// 3. Run through the handler chain
/// 4. Send response(s) and persist updated state
async fn handle_incoming_message(
    config: &HiveConfig,
    store: &Store,
    wa_ctx: &WaMessageContext,
    payment_provider: &Option<Arc<dyn PaymentProvider>>,
) -> Result<bool> {
    use wacore::proto_helpers::MessageExt;

    let sender = wa_ctx.info.source.sender.to_string();
    let is_from_me = wa_ctx.info.source.is_from_me;

    // Skip messages from ourselves
    if is_from_me {
        return Ok(false);
    }

    // Extract text content from the message
    let base_msg = wa_ctx.message.get_base_message();
    let text = base_msg
        .text_content()
        .or_else(|| base_msg.get_caption())
        .unwrap_or("")
        .trim()
        .to_string();

    if text.is_empty() {
        // Handle location messages for orders awaiting location
        let has_location = base_msg.location_message.is_some()
            || base_msg.live_location_message.is_some();

        if !has_location {
            return Ok(false);
        }
    }

    info!("📨 Message from {}: {}", sender, if text.len() > 50 { &text[..50] } else { &text });

    // Load or initialize conversation state
    let mut state = store
        .get_conversation_state(&sender)?
        .map(|json| ConversationState::from_json(&json))
        .unwrap_or_default();

    let is_admin = config.is_admin(&sender);

    // Build our handler context
    let ctx = MessageContext {
        sender: sender.clone(),
        text: text.clone(),
        is_admin,
        is_group: wa_ctx.info.source.is_group,
        has_location: base_msg.location_message.is_some()
            || base_msg.live_location_message.is_some(),
        location_text: extract_location_text(base_msg),
        raw_message: wa_ctx.message.clone(),
        wa_client: wa_ctx.client.clone(),
        chat_jid: wa_ctx.info.source.chat.clone(),
        payment_provider: payment_provider.clone(),
    };

    // Check for cancel/reset commands (but not when in AdminMode — let the admin router handle it)
    if !matches!(state, ConversationState::AdminMode) {
        if text.eq_ignore_ascii_case("cancel")
            || text.eq_ignore_ascii_case("0")
            || text.eq_ignore_ascii_case("home")
            || text.eq_ignore_ascii_case("hi")
            || text.eq_ignore_ascii_case("hello")
        {
            if state.is_in_order_flow() || !matches!(state, ConversationState::Idle) {
                state.reset();
                send_text_reply(&ctx, &config.business.welcome).await?;
                store.save_conversation_state(&sender, &state.to_json())?;
                return Ok(false);
            }
        }
    }

    // Route through handlers
    let result = if is_admin {
        // Try admin handlers first, fall back to regular handlers
        handlers::route_admin_message(config, &ctx, &mut state, store).await?
    } else {
        handlers::route_message(config, &ctx, &mut state, store).await?
    };

    // Send response(s)
    let state_changed = !matches!(result, HandlerResult::NoReply);
    match result {
        HandlerResult::Reply(text) => {
            send_text_reply(&ctx, &text).await?;
        }
        HandlerResult::MultiReply(messages) => {
            for msg in messages {
                send_text_reply(&ctx, &msg).await?;
                // Small delay between messages to maintain order
                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            }
        }
        HandlerResult::NoReply => {}
    }

    // Persist updated conversation state
    store.save_conversation_state(&sender, &state.to_json())?;

    Ok(state_changed)
}

/// Extract a text representation of a location message.
fn extract_location_text(msg: &waproto::whatsapp::Message) -> Option<String> {
    if let Some(ref loc) = msg.location_message {
        let lat = loc.degrees_latitude.unwrap_or(0.0);
        let lng = loc.degrees_longitude.unwrap_or(0.0);
        let name = loc.name.as_deref().unwrap_or("");
        let address = loc.address.as_deref().unwrap_or("");
        if !name.is_empty() || !address.is_empty() {
            Some(format!("{} {} ({}, {})", name, address, lat, lng))
        } else {
            Some(format!("{}, {}", lat, lng))
        }
    } else if let Some(ref loc) = msg.live_location_message {
        let lat = loc.degrees_latitude.unwrap_or(0.0);
        let lng = loc.degrees_longitude.unwrap_or(0.0);
        Some(format!("{}, {}", lat, lng))
    } else {
        None
    }
}

/// Send a simple text reply to the chat.
async fn send_text_reply(ctx: &MessageContext, text: &str) -> Result<()> {
    use waproto::whatsapp as wa;

    let message = wa::Message {
        extended_text_message: Some(Box::new(wa::message::ExtendedTextMessage {
            text: Some(text.to_string()),
            ..Default::default()
        })),
        ..Default::default()
    };

    ctx.wa_client
        .send_message(ctx.chat_jid.clone(), message)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to send message: {}", e))?;

    Ok(())
}
