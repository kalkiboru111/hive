//! M-Pesa payment integration (Safaricom Kenya)
//!
//! Implements STK Push (Lipa na M-Pesa Online) for customer payments.

use super::types::PaymentStatus;
use super::PaymentProvider;
use anyhow::{Result, Context, bail};
use log::{info, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

const MPESA_SANDBOX_URL: &str = "https://sandbox.safaricom.co.ke";
const MPESA_PRODUCTION_URL: &str = "https://api.safaricom.co.ke";

#[derive(Debug, Clone)]
pub struct MpesaConfig {
    pub consumer_key: String,
    pub consumer_secret: String,
    pub shortcode: String,
    pub passkey: String,
    pub callback_url: String,
    pub sandbox: bool,
}

pub struct MpesaClient {
    config: MpesaConfig,
    client: Client,
    access_token: Arc<RwLock<Option<MpesaToken>>>,
}

#[derive(Debug, Clone)]
struct MpesaToken {
    token: String,
    expires_at: std::time::Instant,
}

#[derive(Debug, Deserialize)]
struct AuthResponse {
    access_token: String,
    expires_in: String,
}

#[derive(Debug, Serialize)]
struct StkPushRequest {
    #[serde(rename = "BusinessShortCode")]
    business_short_code: String,
    #[serde(rename = "Password")]
    password: String,
    #[serde(rename = "Timestamp")]
    timestamp: String,
    #[serde(rename = "TransactionType")]
    transaction_type: String,
    #[serde(rename = "Amount")]
    amount: String,
    #[serde(rename = "PartyA")]
    party_a: String,
    #[serde(rename = "PartyB")]
    party_b: String,
    #[serde(rename = "PhoneNumber")]
    phone_number: String,
    #[serde(rename = "CallBackURL")]
    callback_url: String,
    #[serde(rename = "AccountReference")]
    account_reference: String,
    #[serde(rename = "TransactionDesc")]
    transaction_desc: String,
}

#[derive(Debug, Deserialize)]
struct StkPushResponse {
    #[serde(rename = "MerchantRequestID")]
    merchant_request_id: Option<String>,
    #[serde(rename = "CheckoutRequestID")]
    checkout_request_id: Option<String>,
    #[serde(rename = "ResponseCode")]
    response_code: Option<String>,
    #[serde(rename = "ResponseDescription")]
    response_description: Option<String>,
    #[serde(rename = "CustomerMessage")]
    customer_message: Option<String>,
    #[serde(rename = "errorCode")]
    error_code: Option<String>,
    #[serde(rename = "errorMessage")]
    error_message: Option<String>,
}

impl MpesaClient {
    pub fn new(config: MpesaConfig) -> Self {
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
        info!("Fetching new M-Pesa access token");
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

        // Parse expiry (typically "3599")
        let expires_in_secs: u64 = auth_response
            .expires_in
            .parse()
            .unwrap_or(3600);

        let token = MpesaToken {
            token: auth_response.access_token.clone(),
            expires_at: std::time::Instant::now() + std::time::Duration::from_secs(expires_in_secs - 60), // 1 min buffer
        };

        // Cache token
        {
            let mut token_lock = self.access_token.write().await;
            *token_lock = Some(token);
        }

        Ok(auth_response.access_token)
    }

    /// Generate M-Pesa password for STK Push
    fn generate_password(&self, timestamp: &str) -> String {
        let raw = format!("{}{}{}", self.config.shortcode, self.config.passkey, timestamp);
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(raw.as_bytes())
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
}

#[async_trait::async_trait]
impl PaymentProvider for MpesaClient {
    async fn initiate_payment(
        &self,
        amount: f64,
        _currency: &str,
        phone: &str,
        reference: &str,
    ) -> Result<String> {
        let access_token = self.get_access_token().await?;
        let phone_formatted = self.format_phone(phone);
        
        // Generate timestamp (YYYYMMDDHHMMSS)
        let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
        let password = self.generate_password(&timestamp);

        let request = StkPushRequest {
            business_short_code: self.config.shortcode.clone(),
            password,
            timestamp,
            transaction_type: "CustomerPayBillOnline".to_string(),
            amount: amount.round().to_string(),
            party_a: phone_formatted.clone(),
            party_b: self.config.shortcode.clone(),
            phone_number: phone_formatted,
            callback_url: self.config.callback_url.clone(),
            account_reference: reference.to_string(),
            transaction_desc: format!("Payment for order {}", reference),
        };

        let url = format!("{}/mpesa/stkpush/v1/processrequest", self.base_url());
        info!("Initiating M-Pesa STK Push for {} KES to {}", amount, request.phone_number);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .json(&request)
            .send()
            .await
            .context("Failed to send STK Push request")?;

        let status = response.status();
        let response_body: StkPushResponse = response
            .json()
            .await
            .context("Failed to parse STK Push response")?;

        // Check for errors
        if let Some(error_code) = response_body.error_code {
            bail!("M-Pesa error {}: {}", error_code, response_body.error_message.unwrap_or_default());
        }

        if !status.is_success() {
            bail!("M-Pesa STK Push failed ({}): {}", status, response_body.response_description.unwrap_or_default());
        }

        let checkout_request_id = response_body
            .checkout_request_id
            .context("No CheckoutRequestID in response")?;

        info!(
            "✅ M-Pesa STK Push initiated: {} - {}",
            checkout_request_id,
            response_body.customer_message.unwrap_or_default()
        );

        Ok(checkout_request_id)
    }

    async fn check_status(&self, payment_id: &str) -> Result<PaymentStatus> {
        // M-Pesa status check requires the CheckoutRequestID
        // In a real implementation, you'd query the STK Push status endpoint
        // For now, we rely on the callback webhook to update payment status
        warn!("M-Pesa check_status not implemented yet, use webhook callbacks");
        Ok(PaymentStatus::Pending)
    }
}

// Need to add base64 and chrono dependencies
