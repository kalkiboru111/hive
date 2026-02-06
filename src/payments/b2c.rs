//! M-Pesa B2C (Business to Customer) integration for payouts/refunds
//!
//! Allows businesses to send money to customers (e.g., refunds, rewards)

use anyhow::{Result, Context, bail};
use log::{info, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

const MPESA_SANDBOX_URL: &str = "https://sandbox.safaricom.co.ke";
const MPESA_PRODUCTION_URL: &str = "https://api.safaricom.co.ke";

#[derive(Debug, Clone)]
pub struct B2CConfig {
    pub consumer_key: String,
    pub consumer_secret: String,
    pub shortcode: String,
    pub initiator_name: String,
    pub security_credential: String,
    pub callback_url: String,
    pub sandbox: bool,
}

pub struct B2CClient {
    config: B2CConfig,
    client: Client,
    access_token: Arc<RwLock<Option<B2CToken>>>,
}

#[derive(Debug, Clone)]
struct B2CToken {
    token: String,
    expires_at: std::time::Instant,
}

#[derive(Debug, Deserialize)]
struct AuthResponse {
    access_token: String,
    expires_in: String,
}

#[derive(Debug, Serialize)]
struct B2CRequest {
    #[serde(rename = "InitiatorName")]
    initiator_name: String,
    #[serde(rename = "SecurityCredential")]
    security_credential: String,
    #[serde(rename = "CommandID")]
    command_id: String,
    #[serde(rename = "Amount")]
    amount: String,
    #[serde(rename = "PartyA")]
    party_a: String,
    #[serde(rename = "PartyB")]
    party_b: String,
    #[serde(rename = "Remarks")]
    remarks: String,
    #[serde(rename = "QueueTimeOutURL")]
    queue_timeout_url: String,
    #[serde(rename = "ResultURL")]
    result_url: String,
    #[serde(rename = "Occasion")]
    occasion: String,
}

#[derive(Debug, Deserialize)]
struct B2CResponse {
    #[serde(rename = "ConversationID")]
    conversation_id: Option<String>,
    #[serde(rename = "OriginatorConversationID")]
    originator_conversation_id: Option<String>,
    #[serde(rename = "ResponseCode")]
    response_code: Option<String>,
    #[serde(rename = "ResponseDescription")]
    response_description: Option<String>,
    #[serde(rename = "errorCode")]
    error_code: Option<String>,
    #[serde(rename = "errorMessage")]
    error_message: Option<String>,
}

/// B2C transaction type
#[derive(Debug, Clone, Copy)]
pub enum B2CTransactionType {
    /// Business payment to customer
    BusinessPayment,
    /// Salary payment
    SalaryPayment,
    /// Promotion payment (rewards, bonuses)
    PromotionPayment,
}

impl B2CTransactionType {
    fn command_id(&self) -> &str {
        match self {
            B2CTransactionType::BusinessPayment => "BusinessPayment",
            B2CTransactionType::SalaryPayment => "SalaryPayment",
            B2CTransactionType::PromotionPayment => "PromotionPayment",
        }
    }
}

impl B2CClient {
    pub fn new(config: B2CConfig) -> Self {
        Self {
            config,
            client: Client::new(),
            access_token: Arc::new(RwLock::new(None)),
        }
    }

    fn base_url(&self) -> &str {
        if self.config.sandbox {
            MPESA_SANDBOX_URL
        } else {
            MPESA_PRODUCTION_URL
        }
    }

    /// Get OAuth access token (cached)
    async fn get_access_token(&self) -> Result<String> {
        // Check cache first
        {
            let token_lock = self.access_token.read().await;
            if let Some(token) = token_lock.as_ref() {
                if token.expires_at > std::time::Instant::now() {
                    return Ok(token.token.clone());
                }
            }
        }

        // Fetch new token
        info!("Fetching new M-Pesa B2C access token");
        let auth = format!("{}:{}", self.config.consumer_key, self.config.consumer_secret);
        use base64::Engine;
        let auth_base64 = base64::engine::general_purpose::STANDARD.encode(auth.as_bytes());

        let url = format!("{}/oauth/v1/generate?grant_type=client_credentials", self.base_url());
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Basic {}", auth_base64))
            .send()
            .await
            .context("Failed to request M-Pesa access token")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            bail!("M-Pesa auth failed ({}): {}", status, body);
        }

        let auth_response: AuthResponse = response
            .json()
            .await
            .context("Failed to parse M-Pesa auth response")?;

        let expires_in_secs: u64 = auth_response
            .expires_in
            .parse()
            .unwrap_or(3600);

        let token = B2CToken {
            token: auth_response.access_token.clone(),
            expires_at: std::time::Instant::now() + std::time::Duration::from_secs(expires_in_secs - 60),
        };

        // Cache token
        {
            let mut token_lock = self.access_token.write().await;
            *token_lock = Some(token);
        }

        Ok(auth_response.access_token)
    }

    /// Format phone number for M-Pesa (254XXXXXXXXX)
    fn format_phone(&self, phone: &str) -> String {
        let cleaned = phone.replace(&['+', ' ', '-'][..], "");
        if cleaned.starts_with("0") {
            format!("254{}", &cleaned[1..])
        } else if cleaned.starts_with("254") {
            cleaned
        } else {
            format!("254{}", cleaned)
        }
    }

    /// Send money to a customer (B2C payout)
    pub async fn send_payout(
        &self,
        amount: f64,
        phone: &str,
        remarks: &str,
        occasion: &str,
        transaction_type: B2CTransactionType,
    ) -> Result<String> {
        let access_token = self.get_access_token().await?;
        let phone_formatted = self.format_phone(phone);

        let request = B2CRequest {
            initiator_name: self.config.initiator_name.clone(),
            security_credential: self.config.security_credential.clone(),
            command_id: transaction_type.command_id().to_string(),
            amount: amount.round().to_string(),
            party_a: self.config.shortcode.clone(),
            party_b: phone_formatted.clone(),
            remarks: remarks.to_string(),
            queue_timeout_url: self.config.callback_url.clone(),
            result_url: self.config.callback_url.clone(),
            occasion: occasion.to_string(),
        };

        let url = format!("{}/mpesa/b2c/v1/paymentrequest", self.base_url());
        info!("Initiating M-Pesa B2C payout: {} KES to {}", amount, phone_formatted);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .json(&request)
            .send()
            .await
            .context("Failed to send B2C request")?;

        let status = response.status();
        let response_body: B2CResponse = response
            .json()
            .await
            .context("Failed to parse B2C response")?;

        // Check for errors
        if let Some(error_code) = response_body.error_code {
            bail!("M-Pesa B2C error {}: {}", error_code, response_body.error_message.unwrap_or_default());
        }

        if !status.is_success() {
            bail!("M-Pesa B2C failed ({}): {}", status, response_body.response_description.unwrap_or_default());
        }

        let conversation_id = response_body
            .conversation_id
            .context("No ConversationID in response")?;

        info!(
            "✅ M-Pesa B2C payout initiated: {} - {}",
            conversation_id,
            response_body.response_description.unwrap_or_default()
        );

        Ok(conversation_id)
    }

    /// Refund a payment to a customer
    pub async fn refund_payment(
        &self,
        amount: f64,
        phone: &str,
        order_id: i64,
    ) -> Result<String> {
        self.send_payout(
            amount,
            phone,
            &format!("Refund for order #{}", order_id),
            &format!("Order {}", order_id),
            B2CTransactionType::BusinessPayment,
        ).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_phone() {
        let config = B2CConfig {
            consumer_key: "test".to_string(),
            consumer_secret: "test".to_string(),
            shortcode: "600000".to_string(),
            initiator_name: "test".to_string(),
            security_credential: "test".to_string(),
            callback_url: "https://example.com/callback".to_string(),
            sandbox: true,
        };
        let client = B2CClient::new(config);
        
        assert_eq!(client.format_phone("0722000000"), "254722000000");
        assert_eq!(client.format_phone("254722000000"), "254722000000");
        assert_eq!(client.format_phone("+254722000000"), "254722000000");
    }
}
