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
    },
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
        Commands::Init { path } => cmd_init(&path)?,
        Commands::Run { path, phone } => cmd_run(&path, phone).await?,
        Commands::Dashboard { path } => cmd_dashboard(&path).await?,
    }

    Ok(())
}

/// `hive init <path>` — create project scaffold
fn cmd_init(path: &PathBuf) -> Result<()> {
    if path.exists() {
        anyhow::bail!("Directory {} already exists", path.display());
    }

    std::fs::create_dir_all(path)
        .with_context(|| format!("Failed to create directory {}", path.display()))?;

    let config_path = path.join("config.yaml");
    std::fs::write(&config_path, DEFAULT_CONFIG)
        .with_context(|| format!("Failed to write {}", config_path.display()))?;

    // Create data directory for SQLite
    let data_dir = path.join("data");
    std::fs::create_dir_all(&data_dir)?;

    info!("🐝 Created new Hive bot project at {}", path.display());
    info!("   Edit {}/config.yaml to configure your bot", path.display());
    info!("   Then run: hive run {}/", path.display());

    println!("\n🐝 Hive bot project created at {}", path.display());
    println!("\nNext steps:");
    println!("  1. Edit {}/config.yaml", path.display());
    println!("  2. Run: hive run {}/", path.display());
    println!("  3. Scan the QR code with WhatsApp");

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

    // Start dashboard in background if enabled
    let dashboard_handle = if config.dashboard.enabled {
        let dashboard_config = config.clone();
        let dashboard_store = store.clone();
        Some(tokio::spawn(async move {
            if let Err(e) = dashboard::run_dashboard(dashboard_config, dashboard_store).await {
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

    dashboard::run_dashboard(config, store).await
}
