//! Order flow handler.
//!
//! Manages the full order lifecycle:
//! 1. User selects items from menu (by number, supports "1,3,5" or "1")
//! 2. User reviews order summary and confirms
//! 3. User sends delivery location
//! 4. Order is saved, admin is notified

use super::{HandlerResult, MessageContext, MessageHandler};
use crate::bot::conversation::{ConversationState, Order, OrderItem};
use crate::config::{HiveConfig, MessageTemplates};
use crate::store::Store;
use anyhow::Result;
use async_trait::async_trait;

pub struct OrderHandler;

#[async_trait]
impl MessageHandler for OrderHandler {
    fn matches(&self, _text: &str, state: &ConversationState) -> bool {
        matches!(
            state,
            ConversationState::ViewingMenu
                | ConversationState::BuildingOrder(_)
                | ConversationState::ConfirmingOrder(_)
                | ConversationState::AwaitingLocation(_)
        )
    }

    async fn handle(
        &self,
        config: &HiveConfig,
        ctx: &MessageContext,
        state: &mut ConversationState,
        store: &Store,
    ) -> Result<HandlerResult> {
        let text = ctx.text.trim();

        match state.clone() {
            ConversationState::ViewingMenu => {
                handle_item_selection(config, ctx, state, text)
            }
            ConversationState::BuildingOrder(cart) => {
                handle_building_order(config, ctx, state, &cart, text)
            }
            ConversationState::ConfirmingOrder(order) => {
                handle_order_confirmation(config, ctx, state, order, text, store)
            }
            ConversationState::AwaitingLocation(order) => {
                handle_location_input(config, ctx, state, order, text, store).await
            }
            _ => Ok(HandlerResult::NoReply),
        }
    }
}

/// Parse item selections like "1", "1,3,5", "1 3 5", or "2x1" (2 of item 1).
fn parse_item_selections(text: &str) -> Vec<(usize, u32)> {
    let mut selections = Vec::new();

    // Split by comma, space, or newline
    let parts: Vec<&str> = text
        .split(|c: char| c == ',' || c == ' ' || c == '\n')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    for part in parts {
        // Check for "2x1" format (quantity x item)
        if let Some((qty_str, idx_str)) = part.split_once('x') {
            if let (Ok(qty), Ok(idx)) = (qty_str.parse::<u32>(), idx_str.parse::<usize>()) {
                if qty > 0 && idx > 0 {
                    selections.push((idx, qty));
                }
            }
        } else if let Ok(idx) = part.parse::<usize>() {
            if idx > 0 {
                selections.push((idx, 1));
            }
        }
    }

    selections
}

/// Handle item selection from the menu.
fn handle_item_selection(
    config: &HiveConfig,
    _ctx: &MessageContext,
    state: &mut ConversationState,
    text: &str,
) -> Result<HandlerResult> {
    let available = config.available_menu();
    let selections = parse_item_selections(text);

    if selections.is_empty() {
        return Ok(HandlerResult::Reply(
            "Please reply with item number(s) to order.\ne.g. *1* or *1,3,5* or *2x1* (2 of item 1)\n\nReply *0* to go back."
                .to_string(),
        ));
    }

    let mut cart = Vec::new();
    let mut invalid = Vec::new();

    for (idx, qty) in &selections {
        if *idx > 0 && *idx <= available.len() {
            let item = &available[idx - 1];
            cart.push(OrderItem {
                name: item.name.clone(),
                price: item.price,
                quantity: *qty,
                emoji: item.emoji.clone(),
            });
        } else {
            invalid.push(idx.to_string());
        }
    }

    if cart.is_empty() {
        return Ok(HandlerResult::Reply(format!(
            "❌ Invalid item number(s): {}. Please check the menu and try again.",
            invalid.join(", ")
        )));
    }

    let currency = &config.business.currency;
    let delivery_fee = config.delivery.as_ref().map(|d| d.fee).unwrap_or(0.0);

    // Build order summary
    let subtotal: f64 = cart.iter().map(|i| i.subtotal()).sum();
    let total = subtotal + delivery_fee;

    let mut lines = vec!["🛒 *Your Order:*\n".to_string()];
    for item in &cart {
        lines.push(format!("  {}", item.display(currency)));
    }
    lines.push(format!("\nSubtotal: {}{:.2}", currency, subtotal));
    if delivery_fee > 0.0 {
        lines.push(format!("Delivery: {}{:.2}", currency, delivery_fee));
    }
    lines.push(format!("*Total: {}{:.2}*", currency, total));
    lines.push("\n━━━━━━━━━━━━━━━━━━━".to_string());
    lines.push("Reply *YES* to confirm".to_string());
    lines.push("Reply *ADD* + numbers to add more items".to_string());
    lines.push("Reply *0* to cancel".to_string());

    if !invalid.is_empty() {
        lines.push(format!(
            "\n⚠️ Skipped invalid items: {}",
            invalid.join(", ")
        ));
    }

    let order = Order::from_cart(cart, delivery_fee);
    *state = ConversationState::ConfirmingOrder(order);

    Ok(HandlerResult::Reply(lines.join("\n")))
}

/// Handle modifications while building an order.
fn handle_building_order(
    config: &HiveConfig,
    _ctx: &MessageContext,
    state: &mut ConversationState,
    cart: &[OrderItem],
    text: &str,
) -> Result<HandlerResult> {
    let available = config.available_menu();

    // If they type a number, add to cart
    let selections = parse_item_selections(text);
    if !selections.is_empty() {
        let mut new_cart = cart.to_vec();
        for (idx, qty) in &selections {
            if *idx > 0 && *idx <= available.len() {
                let item = &available[idx - 1];
                // Check if already in cart, increase quantity
                if let Some(existing) = new_cart.iter_mut().find(|i| i.name == item.name) {
                    existing.quantity += qty;
                } else {
                    new_cart.push(OrderItem {
                        name: item.name.clone(),
                        price: item.price,
                        quantity: *qty,
                        emoji: item.emoji.clone(),
                    });
                }
            }
        }

        let delivery_fee = config.delivery.as_ref().map(|d| d.fee).unwrap_or(0.0);
        let order = Order::from_cart(new_cart, delivery_fee);
        *state = ConversationState::ConfirmingOrder(order.clone());

        let currency = &config.business.currency;
        let mut lines = vec!["🛒 *Updated Order:*\n".to_string()];
        for item in &order.items {
            lines.push(format!("  {}", item.display(currency)));
        }
        lines.push(format!("\n*Total: {}{:.2}*", currency, order.total));
        lines.push("\nReply *YES* to confirm or *0* to cancel".to_string());

        return Ok(HandlerResult::Reply(lines.join("\n")));
    }

    Ok(HandlerResult::Reply(
        "Reply with item numbers to add, or *0* to cancel.".to_string(),
    ))
}

/// Handle order confirmation (YES/NO).
fn handle_order_confirmation(
    config: &HiveConfig,
    _ctx: &MessageContext,
    state: &mut ConversationState,
    order: Order,
    text: &str,
    _store: &Store,
) -> Result<HandlerResult> {
    let upper = text.to_uppercase();

    if upper == "YES" || upper == "Y" || upper == "CONFIRM" {
        // Move to location phase
        *state = ConversationState::AwaitingLocation(order);

        let delivery_msg = if config.delivery.is_some() {
            "📍 Great! Now send your *delivery address* or share your *location*."
        } else {
            "📍 Please send your address for the order."
        };

        return Ok(HandlerResult::Reply(delivery_msg.to_string()));
    }

    if upper.starts_with("ADD") {
        // Go back to adding items
        let cart = order.items;
        *state = ConversationState::BuildingOrder(cart);
        return Ok(HandlerResult::Reply(
            "📋 Send item number(s) to add to your order:".to_string(),
        ));
    }

    // Show summary again
    let currency = &config.business.currency;
    Ok(HandlerResult::Reply(format!(
        "🛒 Your order total: {}{:.2}\n\nReply *YES* to confirm or *0* to cancel.",
        currency, order.total
    )))
}

/// Handle location input for a confirmed order.
async fn handle_location_input(
    config: &HiveConfig,
    ctx: &MessageContext,
    state: &mut ConversationState,
    mut order: Order,
    text: &str,
    store: &Store,
) -> Result<HandlerResult> {
    // Accept location from location message or text
    let location = if let Some(ref loc) = ctx.location_text {
        loc.clone()
    } else if !text.is_empty() {
        text.to_string()
    } else {
        return Ok(HandlerResult::Reply(
            "📍 Please send your delivery address or share your location.".to_string(),
        ));
    };

    order.location = Some(location.clone());

    // Save order to database
    let items_json = serde_json::to_string(&order.items)?;
    let order_id = store.create_order(
        &ctx.sender,
        &items_json,
        order.subtotal,
        order.delivery_fee,
        order.total,
        None,
    )?;

    // Set location and confirm
    store.set_order_location(order_id, &location)?;

    // Build confirmation message for customer
    let estimate = config
        .delivery
        .as_ref()
        .map(|d| d.estimate_string())
        .unwrap_or_else(|| "30-45 minutes".to_string());

    let customer_msg = MessageTemplates::render(
        &config.messages.order_confirmed,
        &[
            ("id", &order_id.to_string()),
            ("estimate", &estimate),
        ],
    );

    // Build notification for admin(s)
    let currency = &config.business.currency;
    let items_display = order.items_display(currency);
    let admin_msg = MessageTemplates::render(
        &config.messages.order_received_admin,
        &[
            ("id", &order_id.to_string()),
            ("items", &items_display),
            ("currency", currency),
            ("total", &format!("{:.2}", order.total)),
            ("location", &location),
        ],
    );

    // Send admin notification via WhatsApp
    for admin_number in &config.admin_numbers {
        let clean_number: String = admin_number.chars().filter(|c| c.is_ascii_digit()).collect();
        if !clean_number.is_empty() {
            let admin_jid = wacore_binary::jid::Jid::pn(&clean_number);
            let admin_wa_msg = waproto::whatsapp::Message {
                extended_text_message: Some(Box::new(
                    waproto::whatsapp::message::ExtendedTextMessage {
                        text: Some(admin_msg.clone()),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            };
            if let Err(e) = ctx.wa_client.send_message(admin_jid, admin_wa_msg).await {
                log::error!("Failed to notify admin {}: {}", admin_number, e);
            } else {
                log::info!("📢 Notified admin {} about order #{}", admin_number, order_id);
            }
        }
    }

    log::info!(
        "📦 New order #{} from {} — {}{:.2} — {}",
        order_id,
        ctx.sender,
        currency,
        order.total,
        location
    );

    // Reset conversation state
    *state = ConversationState::Idle;

    Ok(HandlerResult::Reply(customer_msg))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_item() {
        let selections = parse_item_selections("1");
        assert_eq!(selections, vec![(1, 1)]);
    }

    #[test]
    fn test_parse_multiple_items() {
        let selections = parse_item_selections("1,3,5");
        assert_eq!(selections, vec![(1, 1), (3, 1), (5, 1)]);
    }

    #[test]
    fn test_parse_quantity_format() {
        let selections = parse_item_selections("2x1");
        assert_eq!(selections, vec![(1, 2)]);
    }

    #[test]
    fn test_parse_mixed() {
        let selections = parse_item_selections("1, 2x3, 5");
        assert_eq!(selections, vec![(1, 1), (3, 2), (5, 1)]);
    }

    #[test]
    fn test_parse_invalid() {
        let selections = parse_item_selections("abc");
        assert!(selections.is_empty());
    }
}
