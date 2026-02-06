#![allow(dead_code)]
//! Hive — WhatsApp Bot Framework for Reality Network
//!
//! CLI entry point with three commands:
//! - `hive init <path>` — scaffold a new bot project
//! - `hive run <path>` — start bot + optional dashboard
//! - `hive dashboard <path>` — start only the dashboard

mod bot;
mod config;
mod dashboard;
mod handlers;
mod i18n;
pub mod network;
mod payments;
mod store;
mod vouchers;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use log::info;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "hive",
    version,
    about = "🐝 Hive — WhatsApp bot framework for Reality Network"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new bot project with default config
    Init {
        /// Directory to create the project in
        path: PathBuf,
        /// Use a template (food-delivery, salon-booking, etc.)
        #[arg(long)]
        template: Option<String>,
    },
    /// Interactive setup wizard for new bots
    Wizard {
        /// Directory to create the project in
        path: PathBuf,
    },
    /// List available templates
    Templates,
    /// Start the bot (and dashboard if enabled)
    Run {
        /// Path to the bot project directory (containing config.yaml)
        path: PathBuf,
        /// Phone number for pair code auth (e.g. +34661479804). If omitted, shows QR code.
        #[arg(long)]
        phone: Option<String>,
    },
    /// Start only the admin dashboard
    Dashboard {
        /// Path to the bot project directory (containing config.yaml)
        path: PathBuf,
    },
}

/// Default config template embedded at compile time
const DEFAULT_CONFIG: &str = include_str!("../templates/default.yaml");

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path, template } => cmd_init(&path, template.as_deref())?,
        Commands::Wizard { path } => cmd_wizard(&path)?,
        Commands::Templates => cmd_templates()?,
        Commands::Run { path, phone } => cmd_run(&path, phone).await?,
        Commands::Dashboard { path } => cmd_dashboard(&path).await?,
    }

    Ok(())
}

/// `hive init <path>` — create project scaffold
fn cmd_init(path: &PathBuf, template: Option<&str>) -> Result<()> {
    if path.exists() {
        anyhow::bail!("Directory {} already exists", path.display());
    }

    std::fs::create_dir_all(path)
        .with_context(|| format!("Failed to create directory {}", path.display()))?;

    // Choose template content
    let config_content = if let Some(template_name) = template {
        load_template(template_name)?
    } else {
        DEFAULT_CONFIG.to_string()
    };

    let config_path = path.join("config.yaml");
    std::fs::write(&config_path, config_content)
        .with_context(|| format!("Failed to write {}", config_path.display()))?;

    // Create data directory for SQLite
    let data_dir = path.join("data");
    std::fs::create_dir_all(&data_dir)?;

    info!("🐝 Created new Hive bot project at {}", path.display());
    info!("   Edit {}/config.yaml to configure your bot", path.display());
    info!("   Then run: hive run {}/", path.display());

    println!("\n🐝 Hive bot project created at {}", path.display());
    if let Some(t) = template {
        println!("   Template: {}", t);
    }
    println!("\nNext steps:");
    println!("  1. Edit {}/config.yaml", path.display());
    println!("  2. Run: hive run {}/", path.display());
    println!("  3. Scan the QR code with WhatsApp");

    Ok(())
}

/// Load template by name (embedded at compile time)
fn load_template(name: &str) -> Result<String> {
    let content = match name {
        "food-delivery" => include_str!("../templates/food-delivery.yaml"),
        "salon-booking" => include_str!("../templates/salon-booking.yaml"),
        "event-tickets" => include_str!("../templates/event-tickets.yaml"),
        "tutoring" => include_str!("../templates/tutoring.yaml"),
        "voucher-store" => include_str!("../templates/voucher-store.yaml"),
        "community-store" => include_str!("../templates/community-store.yaml"),
        "customer-support" => include_str!("../templates/customer-support.yaml"),
        "real-estate" => include_str!("../templates/real-estate.yaml"),
        _ => anyhow::bail!("Unknown template '{}'. Run 'hive templates' to see available templates.", name),
    };
    Ok(content.to_string())
}

/// `hive templates` — list available templates
fn cmd_templates() -> Result<()> {
    println!("🐝 Available Hive Templates:\n");
    println!("  food-delivery      🍔 Restaurant, street food, home kitchen");
    println!("  salon-booking      💇 Hair salon, barber, spa, nails");
    println!("  event-tickets      🎟️  Concerts, workshops, classes, meetups");
    println!("  tutoring           📚 Private lessons, test prep, language learning");
    println!("  voucher-store      🎁 Gift cards, loyalty programs, prepaid credits");
    println!("  community-store    🌾 Co-op, farmer's market, local goods");
    println!("  customer-support   🆘 Help desk, ticket system");
    println!("  real-estate        🏡 Property listings, rental viewings");
    println!("\nUsage:");
    println!("  hive init --template food-delivery my-restaurant");
    println!("  hive init --template salon-booking my-salon");
    println!("\nOr use the wizard for interactive setup:");
    println!("  hive wizard my-business");
    Ok(())
}

/// `hive wizard <path>` — interactive setup
fn cmd_wizard(path: &PathBuf) -> Result<()> {
    use std::io::{self, Write};

    if path.exists() {
        anyhow::bail!("Directory {} already exists", path.display());
    }

    println!("🐝 Hive Setup Wizard\n");
    println!("Let's build your WhatsApp bot! Answer a few questions:\n");

    // Step 1: Business type (with validation)
    println!("1. What type of business are you building?\n");
    println!("   1. Food delivery");
    println!("   2. Salon / Beauty booking");
    println!("   3. Event tickets");
    println!("   4. Tutoring / Lessons");
    println!("   5. Voucher / Gift card store");
    println!("   6. Community store");
    println!("   7. Customer support");
    println!("   8. Real estate");
    println!("   9. Custom (blank template)");

    let template = loop {
        print!("\nYour choice (1-9): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim();

        match choice {
            "1" => break "food-delivery",
            "2" => break "salon-booking",
            "3" => break "event-tickets",
            "4" => break "tutoring",
            "5" => break "voucher-store",
            "6" => break "community-store",
            "7" => break "customer-support",
            "8" => break "real-estate",
            "9" => break "default",
            _ => println!("❌ Invalid choice '{}'. Please enter a number between 1 and 9.", choice),
        }
    };

    // Step 2: Business name (with validation)
    let business_name = loop {
        print!("\n2. What's your business name? ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let name = input.trim();

        if name.is_empty() {
            println!("❌ Business name cannot be empty.");
            continue;
        }
        if name.len() > 50 {
            println!("❌ Business name too long (max 50 characters).");
            continue;
        }
        break name.to_string();
    };

    // Step 3: Currency (with validation)
    let currency = loop {
        print!("\n3. What currency do you use? (USD, EUR, KES, ZAR, etc.) ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let curr = input.trim().to_uppercase();

        if curr.is_empty() {
            println!("❌ Currency cannot be empty.");
            continue;
        }
        if curr.len() != 3 {
            println!("❌ Currency code must be exactly 3 letters (e.g., USD, EUR, KES).");
            continue;
        }
        if !curr.chars().all(|c| c.is_ascii_alphabetic()) {
            println!("❌ Currency code must contain only letters.");
            continue;
        }
        break curr;
    };

    // Step 4: Admin number (with validation)
    let admin_number = loop {
        print!("\n4. Your WhatsApp number (with country code, e.g. +254712345678): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let number = input.trim();

        if number.is_empty() {
            println!("❌ Phone number cannot be empty.");
            continue;
        }
        if !number.starts_with('+') {
            println!("❌ Phone number must start with + (country code).");
            continue;
        }
        if number.len() < 8 || number.len() > 20 {
            println!("❌ Phone number length invalid (expected 8-20 characters including +).");
            continue;
        }
        if !number[1..].chars().all(|c| c.is_ascii_digit()) {
            println!("❌ Phone number must contain only digits after the +.");
            continue;
        }
        break number.to_string();
    };

    // Create project directory
    std::fs::create_dir_all(path)?;
    let data_dir = path.join("data");
    std::fs::create_dir_all(&data_dir)?;

    // Load template and customize
    let mut config_content = if template == "default" {
        DEFAULT_CONFIG.to_string()
    } else {
        load_template(template)?
    };

    // Replace placeholders
    config_content = config_content.replace("My Business", &business_name);
    config_content = config_content.replace("My Kitchen", &business_name);
    config_content = config_content.replace("My Salon", &business_name);
    config_content = config_content.replace("My Events", &business_name);
    config_content = config_content.replace("My Tutoring", &business_name);
    config_content = config_content.replace("My Vouchers", &business_name);
    config_content = config_content.replace("Community Market", &business_name);
    config_content = config_content.replace("Support Bot", &business_name);
    config_content = config_content.replace("Property Listings", &business_name);
    config_content = config_content.replace("\"USD\"", &format!("\"{}\"", currency));
    config_content = config_content.replace("\"+1234567890\"", &format!("\"{}\"", admin_number));

    let config_path = path.join("config.yaml");
    std::fs::write(&config_path, config_content)?;

    println!("\n✅ Bot created at {}", path.display());
    println!("\nNext steps:");
    println!("  1. Review & edit: {}/config.yaml", path.display());
    println!("  2. Run: hive run {}/", path.display());
    println!("  3. Scan the QR code with WhatsApp\n");
    println!("🐝 Your bot is ready to go!");

    Ok(())
}

/// `hive run <path>` — load config, start bot + dashboard
async fn cmd_run(path: &PathBuf, phone: Option<String>) -> Result<()> {
    let config = config::HiveConfig::load(path)
        .with_context(|| format!("Failed to load config from {}", path.display()))?;

    info!("🐝 Starting Hive bot for \"{}\"", config.business.name);

    // Initialize SQLite store
    let db_path = path.join("data").join("hive.db");
    std::fs::create_dir_all(db_path.parent().unwrap())?;
    let store = store::Store::new(db_path.to_str().unwrap())
        .with_context(|| "Failed to initialize database")?;

    // Create shared WhatsApp client (populated after bot connects)
    let wa_client_shared = std::sync::Arc::new(tokio::sync::RwLock::new(None));

    // Start dashboard in background if enabled
    let dashboard_handle = if config.dashboard.enabled {
        let dashboard_config = config.clone();
        let dashboard_store = store.clone();
        let dashboard_client = wa_client_shared.clone();
        Some(tokio::spawn(async move {
            if let Err(e) = dashboard::run_dashboard(dashboard_config, dashboard_store, dashboard_client).await {
                log::error!("Dashboard error: {}", e);
            }
        }))
    } else {
        None
    };

    // Start the WhatsApp bot
    let mut engine = bot::BotEngine::new(config, store, path.clone()).await?;
    if let Some(phone) = phone {
        engine = engine.with_phone_number(phone);
    }
    engine = engine.with_wa_client_shared(wa_client_shared);
    engine.run().await?;

    // Wait for dashboard if it was started
    if let Some(handle) = dashboard_handle {
        handle.await?;
    }

    Ok(())
}

/// `hive dashboard <path>` — start only the dashboard
async fn cmd_dashboard(path: &PathBuf) -> Result<()> {
    let config = config::HiveConfig::load(path)
        .with_context(|| format!("Failed to load config from {}", path.display()))?;

    let db_path = path.join("data").join("hive.db");
    let store = store::Store::new(db_path.to_str().unwrap())
        .with_context(|| "Failed to initialize database")?;

    info!(
        "🐝 Starting Hive dashboard for \"{}\" on port {}",
        config.business.name, config.dashboard.port
    );

    // Dashboard-only mode: no WhatsApp client (webhooks won't send notifications)
    let wa_client_shared = std::sync::Arc::new(tokio::sync::RwLock::new(None));

    dashboard::run_dashboard(config, store, wa_client_shared).await
}
