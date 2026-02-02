//! Message handler trait and routing logic.
//!
//! Each handler implements `MessageHandler` to process a specific type of
//! interaction. The router tries handlers in priority order and dispatches
//! to the first one that matches.

pub mod menu;
pub mod order;
pub mod voucher;

use crate::bot::conversation::ConversationState;
use crate::config::HiveConfig;
use crate::store::Store;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use wacore_binary::jid::Jid;
use whatsapp_rust::client::Client;

/// Context passed to handlers for each incoming message.
pub struct MessageContext {
    /// Sender's phone number / JID string
    pub sender: String,
    /// Extracted text content of the message
    pub text: String,
    /// Whether the sender is an admin
    pub is_admin: bool,
    /// Whether this is a group message
    pub is_group: bool,
    /// Whether the message includes a location
    pub has_location: bool,
    /// Extracted location text (address or coordinates)
    pub location_text: Option<String>,
    /// The raw protobuf message
    pub raw_message: Box<waproto::whatsapp::Message>,
    /// WhatsApp client for sending replies
    pub wa_client: Arc<Client>,
    /// Chat JID to reply to
    pub chat_jid: Jid,
}

/// Result of handling a message.
pub enum HandlerResult {
    /// Send a single text reply.
    Reply(String),
    /// Send multiple text replies in sequence.
    MultiReply(Vec<String>),
    /// No reply needed (already handled or ignored).
    NoReply,
}

/// Trait for message handlers.
#[async_trait]
pub trait MessageHandler: Send + Sync {
    /// Process the message and return a result.
    async fn handle(
        &self,
        config: &HiveConfig,
        ctx: &MessageContext,
        state: &mut ConversationState,
        store: &Store,
    ) -> Result<HandlerResult>;

    /// Check if this handler should process the given message + state.
    fn matches(&self, text: &str, state: &ConversationState) -> bool;
}

/// Route a regular (non-admin) message through the handler chain.
pub async fn route_message(
    config: &HiveConfig,
    ctx: &MessageContext,
    state: &mut ConversationState,
    store: &Store,
) -> Result<HandlerResult> {
    // Skip group messages — only handle DMs
    if ctx.is_group {
        return Ok(HandlerResult::NoReply);
    }

    let text = ctx.text.trim();

    // State-based routing takes priority: if the user is mid-flow,
    // route to the appropriate handler regardless of text content.
    match state {
        ConversationState::BuildingOrder(_) => {
            return order::OrderHandler.handle(config, ctx, state, store).await;
        }
        ConversationState::ConfirmingOrder(_) => {
            return order::OrderHandler.handle(config, ctx, state, store).await;
        }
        ConversationState::AwaitingLocation(_) => {
            return order::OrderHandler.handle(config, ctx, state, store).await;
        }
        ConversationState::ViewingMenu => {
            // If they type a number, treat it as adding an item
            if text.parse::<usize>().is_ok() || text.eq_ignore_ascii_case("order") {
                return order::OrderHandler.handle(config, ctx, state, store).await;
            }
            // Otherwise show menu again or route normally
        }
        ConversationState::RedeemingVoucher => {
            return voucher::VoucherHandler.handle(config, ctx, state, store).await;
        }
        _ => {}
    }

    // Text-based routing for idle state
    match text {
        // Main menu options
        "1" | "menu" => {
            return menu::MenuHandler.handle(config, ctx, state, store).await;
        }
        "2" | "orders" | "my orders" => {
            return handle_my_orders(config, ctx, store).await;
        }
        "3" | "voucher" | "redeem" => {
            *state = ConversationState::RedeemingVoucher;
            return Ok(HandlerResult::Reply(
                "🎟️ Enter your voucher code:".to_string(),
            ));
        }
        "4" | "about" => {
            let about = config
                .business
                .about
                .as_deref()
                .unwrap_or("Thanks for choosing us!");
            return Ok(HandlerResult::Reply(about.to_string()));
        }
        _ => {}
    }

    // Default: show welcome message
    Ok(HandlerResult::Reply(config.business.welcome.clone()))
}

/// Route an admin message. Tries admin commands first, falls back to regular routing.
pub async fn route_admin_message(
    config: &HiveConfig,
    ctx: &MessageContext,
    state: &mut ConversationState,
    store: &Store,
) -> Result<HandlerResult> {
    let text = ctx.text.trim().to_uppercase();

    // Admin commands
    if text.starts_with("DONE ") {
        // Mark order as delivered: "DONE 42"
        if let Ok(order_id) = text[5..].trim().parse::<i64>() {
            return handle_admin_done(config, ctx, store, order_id).await;
        }
    }

    if text.starts_with("VOUCHER ") {
        // Create voucher: "VOUCHER 50" creates a R50 voucher
        if let Ok(amount) = text[8..].trim().parse::<f64>() {
            return handle_admin_create_voucher(config, store, amount).await;
        }
    }

    if text == "ORDERS" || text == "PENDING" {
        return handle_admin_orders(config, store).await;
    }

    if text == "STATS" {
        return handle_admin_stats(config, store).await;
    }

    // Fall through to regular handler chain
    route_message(config, ctx, state, store).await
}

/// Handle "My Orders" — show recent orders for the customer.
async fn handle_my_orders(
    config: &HiveConfig,
    ctx: &MessageContext,
    store: &Store,
) -> Result<HandlerResult> {
    let orders = store.get_customer_orders(&ctx.sender, 5)?;

    if orders.is_empty() {
        return Ok(HandlerResult::Reply(
            "📦 You don't have any orders yet.\n\nReply 1 to view our menu!".to_string(),
        ));
    }

    let currency = &config.business.currency;
    let mut lines = vec!["📦 *Your Recent Orders:*\n".to_string()];

    for order in &orders {
        let status_emoji = match order.status {
            crate::store::OrderStatus::Pending => "⏳",
            crate::store::OrderStatus::Confirmed => "✅",
            crate::store::OrderStatus::Preparing => "🍳",
            crate::store::OrderStatus::Delivering => "🚗",
            crate::store::OrderStatus::Delivered => "🎉",
            crate::store::OrderStatus::Cancelled => "❌",
        };
        lines.push(format!(
            "{} Order #{} — {}{:.2} — {}",
            status_emoji,
            order.id,
            currency,
            order.total,
            order.status.as_str()
        ));
    }

    Ok(HandlerResult::Reply(lines.join("\n")))
}

/// Admin: mark an order as delivered.
async fn handle_admin_done(
    config: &HiveConfig,
    ctx: &MessageContext,
    store: &Store,
    order_id: i64,
) -> Result<HandlerResult> {
    let order = store.get_order(order_id)?;
    match order {
        Some(order) => {
            store.update_order_status(order_id, &crate::store::OrderStatus::Delivered)?;

            // Format the delivery notification for the customer
            let msg = crate::config::MessageTemplates::render(
                &config.messages.order_delivered,
                &[("id", &order_id.to_string())],
            );

            // Send delivery notification to customer via WhatsApp
            let clean_number: String = order.customer_phone
                .chars()
                .filter(|c| c.is_ascii_digit())
                .collect();
            if !clean_number.is_empty() {
                let customer_jid = wacore_binary::jid::Jid::pn(&clean_number);
                let wa_msg = waproto::whatsapp::Message {
                    extended_text_message: Some(Box::new(
                        waproto::whatsapp::message::ExtendedTextMessage {
                            text: Some(msg),
                            ..Default::default()
                        },
                    )),
                    ..Default::default()
                };
                if let Err(e) = ctx.wa_client.send_message(customer_jid, wa_msg).await {
                    log::error!("Failed to notify customer {}: {}", order.customer_phone, e);
                    return Ok(HandlerResult::Reply(format!(
                        "✅ Order #{} marked as delivered.\n⚠️ Failed to notify customer: {}",
                        order_id, e
                    )));
                }
            }

            Ok(HandlerResult::Reply(format!(
                "✅ Order #{} marked as delivered.\n📨 Customer {} has been notified.",
                order_id, order.customer_phone
            )))
        }
        None => Ok(HandlerResult::Reply(format!(
            "❌ Order #{} not found.",
            order_id
        ))),
    }
}

/// Admin: create a voucher.
async fn handle_admin_create_voucher(
    config: &HiveConfig,
    store: &Store,
    amount: f64,
) -> Result<HandlerResult> {
    let code = crate::vouchers::generate_voucher_code();
    store.create_voucher(&code, amount)?;

    let msg = crate::config::MessageTemplates::render(
        &config.messages.voucher_created,
        &[
            ("code", &code),
            ("currency", &config.business.currency),
            ("amount", &format!("{:.2}", amount)),
        ],
    );

    Ok(HandlerResult::Reply(msg))
}

/// Admin: list pending orders.
async fn handle_admin_orders(
    config: &HiveConfig,
    store: &Store,
) -> Result<HandlerResult> {
    let orders = store.list_orders(Some(&crate::store::OrderStatus::Confirmed))?;

    if orders.is_empty() {
        return Ok(HandlerResult::Reply("📋 No pending orders.".to_string()));
    }

    let currency = &config.business.currency;
    let mut lines = vec![format!("📋 *Pending Orders ({} total):*\n", orders.len())];

    for order in &orders {
        let location = order.location.as_deref().unwrap_or("No location");
        lines.push(format!(
            "#{} — {}{:.2} — {}\n📍 {}\nReply: DONE {}",
            order.id, currency, order.total, order.customer_phone, location, order.id
        ));
    }

    Ok(HandlerResult::Reply(lines.join("\n\n")))
}

/// Admin: show stats.
async fn handle_admin_stats(
    config: &HiveConfig,
    store: &Store,
) -> Result<HandlerResult> {
    let stats = store.get_stats()?;
    let currency = &config.business.currency;

    Ok(HandlerResult::Reply(format!(
        "📊 *{} Stats*\n\n\
         📦 Total orders: {}\n\
         ⏳ Active orders: {}\n\
         ✅ Delivered: {}\n\
         💰 Revenue: {}{:.2}\n\
         🎟️ Vouchers: {} created, {} redeemed",
        config.business.name,
        stats.total_orders,
        stats.pending_orders,
        stats.delivered_orders,
        currency,
        stats.total_revenue,
        stats.total_vouchers,
        stats.redeemed_vouchers
    )))
}
