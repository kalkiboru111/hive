//! M-Pesa webhook handler for payment callbacks
//!
//! Receives payment confirmations from Safaricom and updates order status.

use anyhow::Result;
use log::{info, warn};
use serde::{Deserialize, Serialize};

/// M-Pesa callback request structure
#[derive(Debug, Deserialize)]
pub struct MpesaCallback {
    #[serde(rename = "Body")]
    pub body: MpesaCallbackBody,
}

#[derive(Debug, Deserialize)]
pub struct MpesaCallbackBody {
    #[serde(rename = "stkCallback")]
    pub stk_callback: StkCallback,
}

#[derive(Debug, Deserialize)]
pub struct StkCallback {
    #[serde(rename = "MerchantRequestID")]
    pub merchant_request_id: String,
    
    #[serde(rename = "CheckoutRequestID")]
    pub checkout_request_id: String,
    
    #[serde(rename = "ResultCode")]
    pub result_code: i32,
    
    #[serde(rename = "ResultDesc")]
    pub result_desc: String,
    
    #[serde(rename = "CallbackMetadata")]
    pub callback_metadata: Option<CallbackMetadata>,
}

#[derive(Debug, Deserialize)]
pub struct CallbackMetadata {
    #[serde(rename = "Item")]
    pub items: Vec<MetadataItem>,
}

#[derive(Debug, Deserialize)]
pub struct MetadataItem {
    #[serde(rename = "Name")]
    pub name: String,
    
    #[serde(rename = "Value")]
    pub value: serde_json::Value,
}

/// Parsed payment details from callback metadata
#[derive(Debug, Clone, Serialize)]
pub struct PaymentDetails {
    pub amount: f64,
    pub mpesa_receipt_number: String,
    pub transaction_date: String,
    pub phone_number: String,
}

impl StkCallback {
    /// Check if payment was successful
    pub fn is_successful(&self) -> bool {
        self.result_code == 0
    }

    /// Extract payment details from callback metadata
    pub fn parse_payment_details(&self) -> Result<PaymentDetails> {
        let metadata = self.callback_metadata.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No callback metadata"))?;

        let mut amount: Option<f64> = None;
        let mut receipt: Option<String> = None;
        let mut date: Option<String> = None;
        let mut phone: Option<String> = None;

        for item in &metadata.items {
            match item.name.as_str() {
                "Amount" => {
                    amount = Some(item.value.as_f64().unwrap_or_default());
                }
                "MpesaReceiptNumber" => {
                    receipt = item.value.as_str().map(|s| s.to_string());
                }
                "TransactionDate" => {
                    date = Some(item.value.to_string().trim_matches('"').to_string());
                }
                "PhoneNumber" => {
                    phone = Some(item.value.to_string().trim_matches('"').to_string());
                }
                _ => {}
            }
        }

        Ok(PaymentDetails {
            amount: amount.ok_or_else(|| anyhow::anyhow!("Missing amount"))?,
            mpesa_receipt_number: receipt.ok_or_else(|| anyhow::anyhow!("Missing receipt number"))?,
            transaction_date: date.ok_or_else(|| anyhow::anyhow!("Missing transaction date"))?,
            phone_number: phone.ok_or_else(|| anyhow::anyhow!("Missing phone number"))?,
        })
    }
}

/// Process M-Pesa callback and update payment status
pub async fn process_callback(
    callback: MpesaCallback,
    store: &crate::store::Store,
) -> Result<String> {
    let stk = callback.body.stk_callback;
    let checkout_request_id = &stk.checkout_request_id;
    
    info!("📥 M-Pesa callback received: CheckoutRequestID={}, ResultCode={}", 
          checkout_request_id, stk.result_code);

    // Find payment by provider reference (CheckoutRequestID)
    let payment = store.get_payment_by_provider_ref(checkout_request_id)?
        .ok_or_else(|| anyhow::anyhow!("Payment not found for CheckoutRequestID: {}", checkout_request_id))?;

    if stk.is_successful() {
        // Payment successful
        let details = stk.parse_payment_details()?;
        
        info!("✅ M-Pesa payment successful: Receipt={}, Amount={}, Phone={}", 
              details.mpesa_receipt_number, details.amount, details.phone_number);
        
        // Update payment status to completed
        store.update_payment_status(
            &payment.id,
            "completed",
            Some(&stk.checkout_request_id),
        )?;
        
        // Update order status to confirmed
        store.update_order_status(payment.order_id, &crate::store::OrderStatus::Confirmed)?;
        
        info!("💰 Payment {} completed — Order #{} confirmed", payment.id, payment.order_id);
        
        Ok(format!("Payment completed: {}", details.mpesa_receipt_number))
    } else {
        // Payment failed
        warn!("❌ M-Pesa payment failed: ResultCode={}, ResultDesc={}", 
              stk.result_code, stk.result_desc);
        
        store.update_payment_status(
            &payment.id,
            "failed",
            Some(&stk.checkout_request_id),
        )?;
        
        // Optionally update order status to cancelled or leave as pending for cash
        // For now, leave order as-is (customer can pay cash)
        
        Ok(format!("Payment failed: {}", stk.result_desc))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_successful_callback() {
        let json = r#"{
            "Body": {
                "stkCallback": {
                    "MerchantRequestID": "29115-34620561-1",
                    "CheckoutRequestID": "ws_CO_191220191020363925",
                    "ResultCode": 0,
                    "ResultDesc": "The service request is processed successfully.",
                    "CallbackMetadata": {
                        "Item": [
                            {"Name": "Amount", "Value": 1.00},
                            {"Name": "MpesaReceiptNumber", "Value": "NLJ7RT61SV"},
                            {"Name": "TransactionDate", "Value": 20191219102115},
                            {"Name": "PhoneNumber", "Value": 254708374149}
                        ]
                    }
                }
            }
        }"#;

        let callback: MpesaCallback = serde_json::from_str(json).unwrap();
        assert!(callback.body.stk_callback.is_successful());
        
        let details = callback.body.stk_callback.parse_payment_details().unwrap();
        assert_eq!(details.amount, 1.0);
        assert_eq!(details.mpesa_receipt_number, "NLJ7RT61SV");
    }

    #[test]
    fn test_parse_failed_callback() {
        let json = r#"{
            "Body": {
                "stkCallback": {
                    "MerchantRequestID": "29115-34620561-1",
                    "CheckoutRequestID": "ws_CO_191220191020363925",
                    "ResultCode": 1032,
                    "ResultDesc": "Request cancelled by user"
                }
            }
        }"#;

        let callback: MpesaCallback = serde_json::from_str(json).unwrap();
        assert!(!callback.body.stk_callback.is_successful());
        assert_eq!(callback.body.stk_callback.result_desc, "Request cancelled by user");
    }
}
