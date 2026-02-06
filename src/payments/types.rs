//! Payment types and data structures

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: String,
    pub order_id: i64,
    pub amount: f64,
    pub currency: String,
    pub method: PaymentMethod,
    pub status: PaymentStatus,
    pub phone: String,
    pub reference: String,
    pub provider_ref: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PaymentMethod {
    #[serde(rename = "mpesa")]
    MPesa,
    #[serde(rename = "paystack")]
    PayStack,
    #[serde(rename = "stripe")]
    Stripe,
    #[serde(rename = "cash")]
    Cash,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PaymentStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentStatus::Pending => write!(f, "pending"),
            PaymentStatus::Processing => write!(f, "processing"),
            PaymentStatus::Completed => write!(f, "completed"),
            PaymentStatus::Failed => write!(f, "failed"),
            PaymentStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::fmt::Display for PaymentMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentMethod::MPesa => write!(f, "M-Pesa"),
            PaymentMethod::PayStack => write!(f, "PayStack"),
            PaymentMethod::Stripe => write!(f, "Stripe"),
            PaymentMethod::Cash => write!(f, "Cash"),
        }
    }
}
