//! Voucher redemption handler.
//!
//! Processes voucher code input when the user is in `RedeemingVoucher` state.
//! Validates the code, marks it as redeemed, and stores the discount for the
//! next order.

use super::{HandlerResult, MessageContext, MessageHandler};
use crate::bot::conversation::ConversationState;
use crate::config::{HiveConfig, MessageTemplates};
use crate::store::Store;
use anyhow::Result;
use async_trait::async_trait;

pub struct VoucherHandler;

#[async_trait]
impl MessageHandler for VoucherHandler {
    fn matches(&self, _text: &str, state: &ConversationState) -> bool {
        matches!(state, ConversationState::RedeemingVoucher)
    }

    async fn handle(
        &self,
        config: &HiveConfig,
        ctx: &MessageContext,
        state: &mut ConversationState,
        store: &Store,
    ) -> Result<HandlerResult> {
        let code = ctx.text.trim().to_uppercase();

        if code.is_empty() {
            return Ok(HandlerResult::Reply(
                "🎟️ Enter your voucher code:".to_string(),
            ));
        }

        // Try to redeem the voucher
        match store.redeem_voucher(&code, &ctx.sender)? {
            Some(amount) => {
                let currency = &config.business.currency;
                let msg = MessageTemplates::render(
                    &config.messages.voucher_redeemed,
                    &[
                        ("code", &code),
                        ("currency", currency),
                        ("amount", &format!("{:.2}", amount)),
                    ],
                );

                // Reset state
                *state = ConversationState::Idle;

                Ok(HandlerResult::Reply(msg))
            }
            None => {
                let msg = config.messages.voucher_invalid.clone();

                // Check if the voucher exists but was already redeemed
                if let Some(voucher) = store.get_voucher(&code)? {
                    if voucher.redeemed_by.is_some() {
                        *state = ConversationState::Idle;
                        return Ok(HandlerResult::Reply(
                            "❌ This voucher has already been redeemed.".to_string(),
                        ));
                    }
                }

                // Stay in voucher state for retry
                Ok(HandlerResult::Reply(format!(
                    "{}\n\nTry again or reply *0* to go back.",
                    msg
                )))
            }
        }
    }
}
