//! Configuration loading and validation.
//!
//! All bot behavior is driven by a single YAML config file. This module
//! defines the config schema, loads it from disk, and validates it.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Top-level Hive configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveConfig {
    pub business: BusinessConfig,
    pub menu: Vec<MenuItem>,
    #[serde(default)]
    pub delivery: Option<DeliveryConfig>,
    #[serde(default)]
    pub admin_numbers: Vec<String>,
    #[serde(default)]
    pub messages: MessageTemplates,
    #[serde(default)]
    pub dashboard: DashboardConfig,
    #[serde(default)]
    pub network: NetworkConfig,
    #[serde(default)]
    pub payments: PaymentConfig,
}

/// Business identity and messaging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessConfig {
    pub name: String,
    #[serde(default = "default_currency")]
    pub currency: String,
    #[serde(default = "default_welcome")]
    pub welcome: String,
    #[serde(default)]
    pub about: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
}

fn default_currency() -> String {
    "USD".to_string()
}

fn default_welcome() -> String {
    "Welcome! Reply with a number:\n1. 📋 View Menu\n2. 📦 My Orders\n3. 🎟️ Redeem Voucher\n4. ℹ️ About Us".to_string()
}

/// A single menu item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuItem {
    pub name: String,
    pub price: f64,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub emoji: Option<String>,
    #[serde(default = "default_true")]
    pub available: bool,
}

fn default_true() -> bool {
    true
}

/// Delivery configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryConfig {
    #[serde(default)]
    pub fee: f64,
    /// Estimated delivery time range in minutes, e.g. [30, 45]
    #[serde(default)]
    pub estimate_minutes: Option<Vec<u32>>,
    #[serde(default)]
    pub radius_km: Option<f64>,
}

impl DeliveryConfig {
    /// Format the delivery estimate as a human-readable string.
    pub fn estimate_string(&self) -> String {
        match &self.estimate_minutes {
            Some(range) if range.len() == 2 => format!("{}-{} minutes", range[0], range[1]),
            Some(range) if range.len() == 1 => format!("{} minutes", range[0]),
            _ => "30-45 minutes".to_string(),
        }
    }
}

/// Customizable message templates with placeholder support.
///
/// Supported placeholders: `{id}`, `{items}`, `{total}`, `{currency}`,
/// `{location}`, `{estimate}`, `{code}`, `{amount}`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageTemplates {
    #[serde(default = "default_order_confirmed")]
    pub order_confirmed: String,
    #[serde(default = "default_order_received_admin")]
    pub order_received_admin: String,
    #[serde(default = "default_order_delivered")]
    pub order_delivered: String,
    #[serde(default = "default_voucher_created")]
    pub voucher_created: String,
    #[serde(default = "default_voucher_redeemed")]
    pub voucher_redeemed: String,
    #[serde(default = "default_voucher_invalid")]
    pub voucher_invalid: String,
}

impl Default for MessageTemplates {
    fn default() -> Self {
        Self {
            order_confirmed: default_order_confirmed(),
            order_received_admin: default_order_received_admin(),
            order_delivered: default_order_delivered(),
            voucher_created: default_voucher_created(),
            voucher_redeemed: default_voucher_redeemed(),
            voucher_invalid: default_voucher_invalid(),
        }
    }
}

fn default_order_confirmed() -> String {
    "✅ Order #{id} confirmed!\n📍 Send your location or address\n⏱ Estimated delivery: {estimate}"
        .to_string()
}
fn default_order_received_admin() -> String {
    "🔔 New Order #{id}\n{items}\nTotal: {currency}{total}\n📍 {location}\nReply DONE {id} when delivered".to_string()
}
fn default_order_delivered() -> String {
    "🎉 Order #{id} has been delivered! Enjoy your meal!\nRate us: ⭐⭐⭐⭐⭐".to_string()
}
fn default_voucher_created() -> String {
    "🎟️ Voucher created: {code} — {currency}{amount}".to_string()
}
fn default_voucher_redeemed() -> String {
    "✅ Voucher {code} redeemed! {currency}{amount} off your next order.".to_string()
}
fn default_voucher_invalid() -> String {
    "❌ That voucher code is invalid or already used.".to_string()
}

/// Dashboard / admin panel configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Interface to bind. Defaults to loopback (`127.0.0.1`) so the admin panel
    /// is NOT exposed to the network. Set to `0.0.0.0` to expose it — only do
    /// that together with `auth_token` (and ideally a TLS reverse proxy).
    #[serde(default = "default_bind_host")]
    pub bind_host: String,
    /// Optional shared secret. When set, the admin routes require HTTP Basic
    /// auth (any username, password = this token). The M-Pesa webhook callbacks
    /// stay public so the payment provider can reach them.
    #[serde(default)]
    pub auth_token: Option<String>,
}

impl DashboardConfig {
    /// The effective auth token, treating an empty/whitespace value as "no auth
    /// configured". This prevents an empty `auth_token` from enabling the auth
    /// layer while accepting an empty password (a silent bypass).
    pub fn effective_auth_token(&self) -> Option<&str> {
        self.auth_token
            .as_deref()
            .map(str::trim)
            .filter(|t| !t.is_empty())
    }
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            enabled: true,
            bind_host: default_bind_host(),
            auth_token: None,
        }
    }
}

fn default_port() -> u16 {
    8080
}

fn default_bind_host() -> String {
    "127.0.0.1".to_string()
}

/// Reality Network integration configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Whether to submit state snapshots to Reality Network.
    #[serde(default)]
    pub enabled: bool,
    /// Reality testnet L0 node URL.
    #[serde(default = "default_l0_url")]
    pub l0_url: String,
    /// Path to the node identity file (generated on first run).
    #[serde(default = "default_identity_path")]
    pub identity_path: String,
    /// Optional hex-encoded secret key for this rApp's identity.
    ///
    /// Set this to the key you deployed the rApp with (from Reality
    /// `keytool export`) so the deploy address and the snapshot-signing key
    /// match — see docs/DEPLOY.md. If unset, hive generates/loads a key at
    /// `identity_path`.
    #[serde(default)]
    pub identity_key_hex: Option<String>,
    /// Minimum seconds between snapshot submissions (rate limiting).
    #[serde(default = "default_snapshot_interval")]
    pub snapshot_interval_secs: u64,
}

fn default_l0_url() -> String {
    "http://143.110.227.9:9000".to_string()
}

fn default_identity_path() -> String {
    "data/identity.json".to_string()
}

fn default_snapshot_interval() -> u64 {
    30
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            l0_url: default_l0_url(),
            identity_path: default_identity_path(),
            identity_key_hex: None,
            snapshot_interval_secs: default_snapshot_interval(),
        }
    }
}

/// Payment integration configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentConfig {
    /// Whether payments are enabled.
    #[serde(default)]
    pub enabled: bool,
    /// M-Pesa configuration (Kenya).
    #[serde(default)]
    pub mpesa: Option<MpesaConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MpesaConfig {
    pub consumer_key: String,
    pub consumer_secret: String,
    pub shortcode: String,
    pub passkey: String,
    pub callback_url: String,
    #[serde(default)]
    pub sandbox: bool,
}

impl Default for PaymentConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            mpesa: None,
        }
    }
}

impl MessageTemplates {
    /// Render a template string by replacing placeholders.
    pub fn render(template: &str, vars: &[(&str, &str)]) -> String {
        let mut result = template.to_string();
        for (key, value) in vars {
            result = result.replace(&format!("{{{}}}", key), value);
        }
        result
    }
}

impl HiveConfig {
    /// Load config from a directory (looks for `config.yaml` inside it).
    pub fn load(project_dir: &Path) -> Result<Self> {
        let config_path = project_dir.join("config.yaml");
        let contents = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Could not read {}", config_path.display()))?;

        let config: HiveConfig = serde_yaml::from_str(&contents)
            .with_context(|| format!("Invalid YAML in {}", config_path.display()))?;

        config.validate()?;
        Ok(config)
    }

    /// Save the config back to `config.yaml` in the given directory.
    ///
    /// Used by the dashboard onboarding/edit flows to persist business and menu
    /// changes; `config.yaml` remains the source of truth.
    pub fn save(&self, project_dir: &Path) -> Result<()> {
        let config_path = project_dir.join("config.yaml");
        let yaml = serde_yaml::to_string(self).context("Failed to serialize config")?;
        std::fs::write(&config_path, yaml)
            .with_context(|| format!("Failed to write {}", config_path.display()))?;
        Ok(())
    }

    /// Whether the operator has finished setup (business named + at least one
    /// menu item). WhatsApp pairing is tracked separately by the dashboard.
    pub fn setup_complete(&self) -> bool {
        !self.business.name.trim().is_empty() && !self.menu.is_empty()
    }

    /// Validate the config for common mistakes.
    ///
    /// Note: empty `business.name` and empty `menu` are allowed — a brand-new
    /// project boots into the dashboard onboarding flow, which fills them in.
    pub fn validate(&self) -> Result<()> {
        for (i, item) in self.menu.iter().enumerate() {
            if item.name.is_empty() {
                anyhow::bail!("menu[{}].name cannot be empty", i);
            }
            if item.price < 0.0 {
                anyhow::bail!("menu[{}].price cannot be negative", i);
            }
        }
        if self.dashboard.port == 0 {
            anyhow::bail!("dashboard.port must be > 0");
        }
        Ok(())
    }

    /// Check if a phone number is an admin.
    pub fn is_admin(&self, phone: &str) -> bool {
        // Strip non-digits from both sides for comparison
        // Config might have "+14152657184", sender might be "14152657184@s.whatsapp.net"
        let phone_digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
        self.admin_numbers.iter().any(|n| {
            let n_digits: String = n.chars().filter(|c| c.is_ascii_digit()).collect();
            n_digits == phone_digits
        })
    }

    /// Get available menu items only.
    pub fn available_menu(&self) -> Vec<&MenuItem> {
        self.menu.iter().filter(|m| m.available).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_render() {
        let result = MessageTemplates::render(
            "Order #{id} total: {currency}{total}",
            &[("id", "42"), ("currency", "ZAR"), ("total", "105.00")],
        );
        assert_eq!(result, "Order #42 total: ZAR105.00");
    }

    #[test]
    fn test_delivery_estimate_string() {
        let cfg = DeliveryConfig {
            fee: 10.0,
            estimate_minutes: Some(vec![30, 45]),
            radius_km: None,
        };
        assert_eq!(cfg.estimate_string(), "30-45 minutes");
    }
}
