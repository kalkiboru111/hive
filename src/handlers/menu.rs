//! Menu display handler.
//!
//! Formats the menu from config and presents it to the user.
//! Transitions the conversation to `ViewingMenu` state.

use super::{HandlerResult, MessageContext, MessageHandler};
use crate::bot::conversation::ConversationState;
use crate::config::HiveConfig;
use crate::store::Store;
use anyhow::Result;
use async_trait::async_trait;

pub struct MenuHandler;

#[async_trait]
impl MessageHandler for MenuHandler {
    fn matches(&self, text: &str, _state: &ConversationState) -> bool {
        let t = text.trim();
        t == "1" || t.eq_ignore_ascii_case("menu")
    }

    async fn handle(
        &self,
        config: &HiveConfig,
        _ctx: &MessageContext,
        state: &mut ConversationState,
        _store: &Store,
    ) -> Result<HandlerResult> {
        let available = config.available_menu();

        if available.is_empty() {
            return Ok(HandlerResult::Reply(
                "😔 Sorry, our menu is currently empty. Check back later!".to_string(),
            ));
        }

        let currency = &config.business.currency;
        let mut lines = vec![format!("📋 *{} Menu*\n", config.business.name)];

        for (i, item) in available.iter().enumerate() {
            let emoji = item.emoji.as_deref().unwrap_or("•");
            let desc = item
                .description
                .as_deref()
                .map(|d| format!("\n   _{}_", d))
                .unwrap_or_default();

            lines.push(format!(
                "{}. {} *{}* — {}{:.2}{}",
                i + 1,
                emoji,
                item.name,
                currency,
                item.price,
                desc
            ));
        }

        // Add delivery fee info if configured
        if let Some(ref delivery) = config.delivery {
            if delivery.fee > 0.0 {
                lines.push(format!(
                    "\n🚗 Delivery fee: {}{:.2}",
                    currency, delivery.fee
                ));
            }
            lines.push(format!("⏱ Estimated: {}", delivery.estimate_string()));
        }

        lines.push("\n━━━━━━━━━━━━━━━━━━━".to_string());
        lines.push("Reply with item number(s) to order".to_string());
        lines.push("e.g. *1* or *1,3,5*".to_string());
        lines.push("Reply *0* to go back".to_string());

        *state = ConversationState::ViewingMenu;

        Ok(HandlerResult::Reply(lines.join("\n")))
    }
}

/// Format a compact menu summary (used in order confirmations, etc.)
pub fn format_menu_compact(config: &HiveConfig) -> String {
    let available = config.available_menu();
    let currency = &config.business.currency;

    available
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let emoji = item.emoji.as_deref().unwrap_or("•");
            format!("{}. {} {} — {}{:.2}", i + 1, emoji, item.name, currency, item.price)
        })
        .collect::<Vec<_>>()
        .join("\n")
}
