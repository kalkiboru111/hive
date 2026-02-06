# M-Pesa Advanced Features

This guide covers advanced M-Pesa features: admin notifications, payment reconciliation, retry logic, and B2C payouts (refunds).

## Features Overview

### ✅ 1. Admin WhatsApp Notifications

When a payment is completed, all admins listed in `config.yaml` receive a WhatsApp notification with:
- Order number
- Payment amount
- M-Pesa receipt number
- Customer phone number
- Delivery location

**Example notification:**
```
💰 *Payment Received*

Order #123
Amount: KES 500.00
Receipt: NLJ7RT61SV
Customer: +254722000000
Location: Downtown Nairobi

✅ Order confirmed and ready to prepare!
```

**Configuration:**
```yaml
admin_numbers:
  - "+254700000000"  # Admin 1
  - "+254711111111"  # Admin 2
```

**How it works:**
- Webhook receives payment confirmation from Safaricom
- Database updated (payment status → completed, order status → confirmed)
- WhatsApp messages sent to all admins in parallel
- Failures logged but don't block webhook response

---

## ✅ 2. Retry Logic & Idempotency

M-Pesa may send duplicate callbacks due to network issues. Hive handles this gracefully:

**Idempotency check:**
```rust
// If payment already completed, return early
if payment.status == Completed {
    info!("⚠️ Payment already processed (idempotent retry)");
    return Ok("Already processed");
}
```

**Benefits:**
- Prevents double-processing of payments
- Admins don't receive duplicate notifications
- Order status remains consistent
- Safaricom retries don't cause issues

**Safaricom retry behavior:**
- Retries callback up to 3 times if no response received
- Exponential backoff: 1s, 5s, 15s
- After 3 failures, payment marked as "timeout"

**Your responsibility:**
- Ensure webhook endpoint is always available
- Return HTTP 200 within 20 seconds
- Don't perform long-running operations in webhook handler

---

## ✅ 3. Payment Reconciliation Dashboard

### API Endpoints

**List all payments:**
```bash
GET /api/payments

Response:
[
  {
    "id": "PAY-123-1675670400",
    "order_id": 123,
    "amount": 500.0,
    "currency": "KES",
    "method": "mpesa",
    "status": "completed",
    "phone": "254722000000",
    "reference": "Order #123",
    "provider_ref": "ws_CO_191220191020363925",
    "created_at": "2026-02-06 12:00:00",
    "updated_at": "2026-02-06 12:01:30"
  }
]
```

**Get single payment:**
```bash
GET /api/payments/{payment_id}

Response: (same structure as above)
```

**Enhanced stats:**
```bash
GET /api/stats

Response:
{
  "total_orders": 150,
  "pending_orders": 12,
  "delivered_orders": 138,
  "total_revenue": 75000.0,
  "total_payments": 145,
  "completed_payments": 142,
  "failed_payments": 3,
  "payment_revenue": 71000.0
}
```

### Reconciliation Workflow

**1. Export payments for accounting:**
```bash
curl http://localhost:8080/api/payments > payments.json
```

**2. Compare with M-Pesa statement:**
- Download M-Pesa transaction report from Safaricom portal
- Match `provider_ref` (CheckoutRequestID) with report
- Verify amounts match

**3. Identify discrepancies:**
```bash
# Find payments stuck in "processing"
curl http://localhost:8080/api/payments | jq '.[] | select(.status == "processing")'

# Find failed payments
curl http://localhost:8080/api/payments | jq '.[] | select(.status == "failed")'
```

**4. Manual resolution:**
- Contact Safaricom support with `provider_ref`
- Update payment status manually if needed
- Issue refunds for failed transactions

### Reconciliation Best Practices

**Daily:**
- Check `completed_payments` vs `total_orders`
- Verify `payment_revenue` matches expected amount
- Review any "processing" payments older than 24h

**Weekly:**
- Download M-Pesa statement
- Cross-reference receipts with database
- Investigate discrepancies

**Monthly:**
- Full reconciliation with accounting software
- Archive old payment records
- Generate revenue reports

---

## ✅ 4. B2C Payouts (Refunds)

M-Pesa B2C (Business to Customer) allows you to send money back to customers.

### Use Cases
- **Refunds:** Customer cancels order, return payment
- **Rewards:** Loyalty bonuses, referral rewards
- **Compensation:** Service issues, delays

### Configuration

B2C requires **separate credentials** from STK Push:

```yaml
payments:
  enabled: true
  mpesa:
    # STK Push config (existing)
    consumer_key: "YOUR_CONSUMER_KEY"
    consumer_secret: "YOUR_CONSUMER_SECRET"
    shortcode: "174379"
    passkey: "YOUR_PASSKEY"
    callback_url: "https://yourdomain.com/api/mpesa/callback"
    sandbox: true

  b2c:
    # B2C-specific config (NEW)
    consumer_key: "YOUR_B2C_CONSUMER_KEY"
    consumer_secret: "YOUR_B2C_CONSUMER_SECRET"
    shortcode: "600000"  # B2C shortcode (different from STK)
    initiator_name: "testapi"  # From Safaricom portal
    security_credential: "ENCRYPTED_PASSWORD"  # Encrypted initiator password
    callback_url: "https://yourdomain.com/api/mpesa/b2c/callback"
    sandbox: true
```

### Getting B2C Credentials

**Sandbox:**
1. Go to https://developer.safaricom.co.ke
2. Create new app with "B2C" API
3. Get initiator name and password from docs
4. Encrypt password with Safaricom public cert (see below)

**Production:**
1. Contact Safaricom to enable B2C for your business
2. Register initiator credentials
3. Receive B2C shortcode and credentials

### Security Credential Encryption

Safaricom requires the initiator password to be encrypted with their public certificate:

```bash
# Download Safaricom public certificate
wget https://developer.safaricom.co.ke/cert/SandboxCertificate.cer

# Encrypt your password
openssl pkeyutl \
  -encrypt \
  -pubin \
  -inkey <(openssl x509 -in SandboxCertificate.cer -pubkey -noout) \
  -in <(echo -n 'YOUR_PASSWORD') \
  -out encrypted.bin

# Base64 encode the result
base64 -i encrypted.bin

# Use the output as security_credential in config.yaml
```

### API: Refund a Payment

```bash
POST /api/payments/{payment_id}/refund

Response (success):
{
  "success": true,
  "conversation_id": "AG_20260206_...",
  "message": "Refund of KES 500.00 initiated to 254722000000"
}

Response (error):
{
  "error": "M-Pesa B2C (refunds) not configured. Contact admin."
}
```

### Refund Flow

1. **Admin initiates refund** via dashboard API
2. **Hive validates:**
   - Payment exists and is completed
   - B2C is configured
   - Sufficient balance in B2C account
3. **M-Pesa processes:**
   - Deducts from business shortcode
   - Sends to customer phone
   - Sends callback to confirm
4. **Customer receives:**
   - SMS notification from M-Pesa
   - Money in M-Pesa wallet instantly

### B2C Transaction Types

```rust
pub enum B2CTransactionType {
    BusinessPayment,   // General refunds/payments
    SalaryPayment,     // Employee salaries
    PromotionPayment,  // Rewards, bonuses
}
```

**Choose based on use case:**
- **Refunds:** Use `BusinessPayment`
- **Loyalty rewards:** Use `PromotionPayment`
- **Employee payouts:** Use `SalaryPayment`

### B2C Callback Handling

Similar to STK Push, B2C sends callbacks:

```json
{
  "Result": {
    "ResultType": 0,
    "ResultCode": 0,
    "ResultDesc": "The service request is processed successfully.",
    "OriginatorConversationID": "AG_20260206_...",
    "ConversationID": "AG_20260206_...",
    "TransactionID": "OAJ7RT61SV",
    "ResultParameters": {
      "ResultParameter": [
        {"Key": "TransactionAmount", "Value": 500.00},
        {"Key": "TransactionReceipt", "Value": "OAJ7RT61SV"},
        {"Key": "ReceiverPartyPublicName", "Value": "254722000000"},
        {"Key": "TransactionCompletedDateTime", "Value": "06.02.2026 12:30:45"}
      ]
    }
  }
}
```

**TODO:** Implement B2C callback handler (similar to STK Push webhook)

### Refund Limits

**Sandbox:**
- Max per transaction: KES 70,000
- No daily limit
- Test phone numbers only

**Production:**
- Max per transaction: KES 150,000
- Daily limit: Based on your business tier
- All Kenyan M-Pesa numbers

**Fees:**
- B2C transactions incur M-Pesa fees
- Typically KES 10-30 per transaction
- Check Safaricom rates for your business tier

### Best Practices

**1. Verify before refunding:**
```rust
// Check order status
let order = store.get_order(payment.order_id)?;
if order.status == Delivered {
    return Err("Cannot refund delivered order");
}
```

**2. Partial refunds:**
```rust
// Refund only part of payment
let refund_amount = payment.amount * 0.5; // 50% refund
b2c.send_payout(refund_amount, &payment.phone, ...);
```

**3. Record refunds:**
```sql
-- Add refunds table to track
CREATE TABLE refunds (
    id TEXT PRIMARY KEY,
    payment_id TEXT,
    amount REAL,
    conversation_id TEXT,
    status TEXT,
    created_at TEXT
);
```

**4. Customer communication:**
- Notify customer via WhatsApp before refunding
- Explain reason for refund
- Confirm phone number is correct

**5. Fraud prevention:**
- Require admin approval for large refunds
- Log all refund requests with timestamps
- Monitor for abuse patterns

---

## Troubleshooting

### Admin notifications not received

**Check:**
```bash
# Verify admin numbers in config
cat config.yaml | grep -A 5 admin_numbers

# Check logs for notification errors
tail -f hive.log | grep "notify admin"
```

**Common issues:**
- WhatsApp not connected when webhook fires
- Admin number format incorrect (must include country code)
- WhatsApp rate limiting (too many messages too fast)

### Payment stuck in "processing"

**Possible causes:**
1. Webhook not called (Safaricom network issue)
2. Webhook endpoint unreachable
3. Database write failed

**Resolution:**
```bash
# Check payment status in M-Pesa portal
# Manually update if confirmed:
sqlite3 data/hive.db "UPDATE payments SET status='completed' WHERE provider_ref='ws_CO_...';"
```

### B2C refund failed

**Error codes:**
- `2001` - Initiator authentication failed (wrong password)
- `2006` - Insufficient balance
- `2032` - Invalid phone number
- `2033` - Transaction amount too small (min KES 10)

**Check balance:**
```bash
# Contact Safaricom or check B2C portal
# Production: SMS "BAL" to 334
```

---

## Security Checklist

- [ ] Webhook uses HTTPS (production)
- [ ] Webhook validates M-Pesa signature (TODO)
- [ ] Admin numbers kept confidential
- [ ] B2C credentials encrypted at rest
- [ ] Refunds require admin authentication
- [ ] Payment logs archived securely
- [ ] Database backups enabled
- [ ] Rate limiting on refund endpoint
- [ ] Audit trail for all financial operations

---

## Next Steps

1. **Test in sandbox:**
   - Place test orders
   - Trigger webhooks
   - Verify notifications
   - Test refund flow

2. **Monitor in production:**
   - Set up alerts for failed payments
   - Daily reconciliation checks
   - Weekly financial reports

3. **Scale up:**
   - Add payment analytics dashboard
   - Implement automatic reconciliation
   - Build customer refund request form
   - Integrate with accounting software

---

## Support

- **M-Pesa Docs:** https://developer.safaricom.co.ke
- **Hive Issues:** https://github.com/kalkiboru111/hive/issues
- **Safaricom Support:** apisupport@safaricom.co.ke
