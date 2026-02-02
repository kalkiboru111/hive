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
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            enabled: true,
        }
    }
}

fn default_port() -> u16 {
    8080
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

    /// Validate the config for common mistakes.
    pub fn validate(&self) -> Result<()> {
        if self.business.name.is_empty() {
            anyhow::bail!("business.name cannot be empty");
        }
        if self.menu.is_empty() {
            anyhow::bail!("menu must contain at least one item");
        }
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
        self.admin_numbers.iter().any(|n| n == phone)
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
