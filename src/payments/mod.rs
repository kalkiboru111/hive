//! Payment integrations for Hive
//!
//! Supports:
//! - M-Pesa (Kenya) - Mobile money via Safaricom
//! - PayStack (Nigeria, Ghana, South Africa) - Card payments
//! - Stripe (International) - Coming soon

pub mod mpesa;
pub mod types;

pub use mpesa::MpesaClient;
pub use types::{Payment, PaymentMethod, PaymentStatus};

use anyhow::Result;

/// Payment provider trait
#[async_trait::async_trait]
pub trait PaymentProvider: Send + Sync {
    /// Initiate a payment request
    async fn initiate_payment(
        &self,
        amount: f64,
        currency: &str,
        phone: &str,
        reference: &str,
    ) -> Result<String>;

    /// Check payment status
    async fn check_status(&self, payment_id: &str) -> Result<PaymentStatus>;
}
