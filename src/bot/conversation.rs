//! Conversation state machine.
//!
//! Each user (identified by phone number) has a `ConversationState` that
//! tracks where they are in the bot's flow. State transitions happen in
//! handlers and are persisted to SQLite.

use serde::{Deserialize, Serialize};

/// A single item in an order being built.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub name: String,
    pub price: f64,
    pub quantity: u32,
    pub emoji: Option<String>,
}

impl OrderItem {
    pub fn subtotal(&self) -> f64 {
        self.price * self.quantity as f64
    }

    /// Format this item for display, e.g. "2x 🌯 Kota — R70.00"
    pub fn display(&self, currency: &str) -> String {
        let emoji = self.emoji.as_deref().unwrap_or("");
        if self.quantity > 1 {
            format!(
                "{}x {} {} — {}{:.2}",
                self.quantity,
                emoji,
                self.name,
                currency,
                self.subtotal()
            )
        } else {
            format!(
                "{} {} — {}{:.2}",
                emoji, self.name, currency, self.price
            )
        }
    }
}

/// An order that has been confirmed (items + total locked in).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// Database ID (set after insert)
    #[serde(default)]
    pub id: Option<i64>,
    pub items: Vec<OrderItem>,
    pub subtotal: f64,
    pub delivery_fee: f64,
    pub total: f64,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub voucher_discount: f64,
}

impl Order {
    /// Build an order from cart items + delivery fee.
    pub fn from_cart(items: Vec<OrderItem>, delivery_fee: f64) -> Self {
        let subtotal: f64 = items.iter().map(|i| i.subtotal()).sum();
        let total = subtotal + delivery_fee;
        Self {
            id: None,
            items,
            subtotal,
            delivery_fee,
            total,
            location: None,
            voucher_discount: 0.0,
        }
    }

    /// Format order items as a string list.
    pub fn items_display(&self, currency: &str) -> String {
        self.items
            .iter()
            .map(|i| i.display(currency))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Apply a voucher discount.
    pub fn apply_discount(&mut self, amount: f64) {
        self.voucher_discount = amount;
        self.total = (self.subtotal + self.delivery_fee - amount).max(0.0);
    }
}

/// Tracks where a user is in the conversation flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConversationState {
    /// Default state — waiting for user input.
    Idle,

    /// User is viewing the menu and can add items.
    ViewingMenu,

    /// User is building an order (selecting items, adjusting quantities).
    BuildingOrder(Vec<OrderItem>),

    /// User has completed item selection and is reviewing before confirming.
    ConfirmingOrder(Order),

    /// Order confirmed — waiting for delivery location/address.
    AwaitingLocation(Order),

    /// User is entering a voucher code.
    RedeemingVoucher,

    /// Admin mode — admin commands routed by number instead of text prefix.
    AdminMode,
}

impl Default for ConversationState {
    fn default() -> Self {
        Self::Idle
    }
}

impl ConversationState {
    /// Serialize state to JSON for database storage.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| r#""Idle""#.to_string())
    }

    /// Deserialize state from JSON.
    pub fn from_json(json: &str) -> Self {
        serde_json::from_str(json).unwrap_or_default()
    }

    /// Human-readable label for the current state.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::ViewingMenu => "viewing_menu",
            Self::BuildingOrder(_) => "building_order",
            Self::ConfirmingOrder(_) => "confirming_order",
            Self::AwaitingLocation(_) => "awaiting_location",
            Self::RedeemingVoucher => "redeeming_voucher",
            Self::AdminMode => "admin_mode",
        }
    }

    /// Reset to idle state.
    pub fn reset(&mut self) {
        *self = Self::Idle;
    }

    /// Check if the user is mid-order.
    pub fn is_in_order_flow(&self) -> bool {
        matches!(
            self,
            Self::BuildingOrder(_) | Self::ConfirmingOrder(_) | Self::AwaitingLocation(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_item_display() {
        let item = OrderItem {
            name: "Kota".to_string(),
            price: 35.0,
            quantity: 2,
            emoji: Some("🌯".to_string()),
        };
        assert_eq!(item.display("R"), "2x 🌯 Kota — R70.00");
    }

    #[test]
    fn test_order_from_cart() {
        let items = vec![
            OrderItem {
                name: "Kota".to_string(),
                price: 35.0,
                quantity: 1,
                emoji: None,
            },
            OrderItem {
                name: "Gatsby".to_string(),
                price: 60.0,
                quantity: 1,
                emoji: None,
            },
        ];
        let order = Order::from_cart(items, 10.0);
        assert_eq!(order.subtotal, 95.0);
        assert_eq!(order.total, 105.0);
    }

    #[test]
    fn test_state_serialization_roundtrip() {
        let state = ConversationState::BuildingOrder(vec![OrderItem {
            name: "Test".to_string(),
            price: 10.0,
            quantity: 1,
            emoji: None,
        }]);
        let json = state.to_json();
        let restored = ConversationState::from_json(&json);
        assert!(matches!(restored, ConversationState::BuildingOrder(_)));
    }
}
