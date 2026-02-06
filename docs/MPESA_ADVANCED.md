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

---

## ✅ 5. Bank Credit / Loan Applications

**NEW:** Export your complete financial history for bank credit applications!

### Why This Matters

Traditional lenders require proof of business income and history. With Hive on Reality Network, you have:
- **Immutable ledger** — All transactions on-chain, tamper-proof
- **Cryptographic proof** — Reality Network consensus validates your history
- **Comprehensive records** — Orders, payments, refunds, all in one place
- **Monthly breakdowns** — Show growth trajectory to lenders
- **Payment success rates** — Demonstrate customer trust

### Export Ledger

```bash
GET /api/export/ledger

# Downloads: your-business-ledger-20260206.json
```

**What's included:**
- Business information (name, currency, contact)
- Summary statistics (revenue, orders, payments, success rates)
- Monthly revenue breakdown (growth trend)
- All orders with dates and amounts
- All payments with receipts and statuses
- All refunds with reasons
- Reality Network verification proof
- Bank-specific recommendations

**Example output:**
```json
{
  "generated_at": "2026-02-06T12:00:00Z",
  "business": {
    "name": "Cloudy Deliveries",
    "currency": "KES",
    "phone": "+254722000000"
  },
  "summary": {
    "total_revenue": 450000.00,
    "payment_revenue": 432000.00,
    "total_orders": 1250,
    "delivered_orders": 1180,
    "payment_success_rate": "96.80%",
    "refund_rate": "2.10%"
  },
  "monthly_breakdown": [
    {"month": "2025-12", "revenue": 120000, "orders": 350},
    {"month": "2026-01", "revenue": 165000, "orders": 450},
    {"month": "2026-02", "revenue": 165000, "orders": 450}
  ],
  "verification": {
    "platform": "Hive on Reality Network",
    "ledger": "Reality Network L0/L1 Consensus Layer",
    "proof": "All transactions submitted to decentralized ledger",
    "auditable": true,
    "tamper_proof": true,
    "note": "This financial history is backed by cryptographic proofs..."
  },
  "for_bank_use": {
    "purpose": "Credit application / loan assessment",
    "data_accuracy": "Verified via blockchain consensus",
    "recommended_actions": [...]
  }
}
```

### How to Use for Credit Applications

**1. Generate your ledger:**
```bash
curl http://localhost:8080/api/export/ledger > ledger.json
```

**2. Print or convert to PDF:**
```bash
# Convert JSON to readable format
jq '.' ledger.json > ledger-readable.json

# Or use online tools: jsonviewer.stack.hu
```

**3. Submit to bank with these talking points:**
- "All data is backed by Reality Network's decentralized ledger"
- "Transaction history is cryptographically verified"
- "Payment success rate of X% demonstrates customer trust"
- "Monthly growth of Y% shows business viability"
- "You can independently verify M-Pesa receipts"

**4. Key metrics banks care about:**
- **Revenue trend** — Are you growing?
- **Payment success rate** — Do customers trust you?
- **Refund rate** — Are you reliable?
- **Order frequency** — Consistent income?
- **Average order value** — Ticket size

### Real-World Example

**Scenario:** Cloudy needs KES 500,000 loan to buy a commercial oven.

**Her ledger shows:**
- 6 months of trading history
- KES 450,000 total revenue
- 96.8% payment success rate
- 2.1% refund rate (industry avg: 5-8%)
- Monthly growth: 25% MoM
- 1,250 orders from 320 unique customers

**Bank's perspective:**
- ✅ Proven track record (6 months)
- ✅ High payment success (trustworthy)
- ✅ Low refunds (quality service)
- ✅ Growing revenue (viable business)
- ✅ Verifiable data (blockchain proof)

**Result:** Approved for KES 500,000 at 12% interest.

### Tips for Maximizing Approval

**Build history:**
- Trade for at least 3-6 months before applying
- Aim for 200+ orders minimum
- Keep payment success rate >90%

**Show growth:**
- Month-over-month revenue increase
- Expanding customer base
- Increasing order frequency

**Maintain quality:**
- Low refund rate (<5%)
- Fast delivery times
- Positive customer interactions

**Document everything:**
- Save M-Pesa statements (corroborate ledger)
- Keep supplier invoices (prove costs)
- Track expenses (show profitability)

### Alternative Credit Sources

If banks reject you, try:
- **Microfinance institutions** (lower requirements)
- **Peer-to-peer lending** (Tala, Branch, M-Shwari)
- **Supplier credit** (pay after selling)
- **Community savings groups** (chamas)
- **Reality Network DeFi** (coming soon — collateralized $NET loans)

---

## ✅ 6. Payment Analytics

Real-time insights into your payment performance.

```bash
GET /api/analytics/payments
```

**Metrics provided:**
- Daily revenue time series (last 30 days)
- Payment method breakdown
- Average order value
- Transaction volume trends
- Success/failure rates

**Use cases:**
- Spot revenue drops quickly
- Identify best-selling days/times
- Track customer spending patterns
- Monitor payment health

---

## ✅ 7. Automatic Reconciliation

Daily health check for your financial records.

```bash
GET /api/reconciliation/report
```

**What it checks:**
- Revenue vs payments (should match)
- Orders without payments (investigate)
- High failure rates (fix issues)
- Pending refunds (monitor B2C callbacks)
- Stuck payments (processing >24h)

**Example report:**
```json
{
  "status": "needs_review",
  "summary": {
    "total_revenue": 450000,
    "payment_revenue": 432000,
    "total_refunded": 18000,
    "net_revenue": 432000
  },
  "health_checks": {
    "payment_success_rate": "96.8%",
    "payment_failure_rate": "3.2%",
    "refund_completion_rate": "100.0%"
  },
  "issues": [
    {
      "severity": "warning",
      "issue": "12 orders without payment records",
      "action": "Review cash orders or missing payment data"
    }
  ]
}
```

**Best practice:** Run reconciliation daily, review issues weekly.

---

## Testing

Run the comprehensive test suite:

```bash
cd hive
./test_advanced_features.sh
```

**Tests cover:**
1. ✅ Stats API
2. ✅ Payments list
3. ✅ Payment analytics
4. ✅ Reconciliation report
5. ✅ Refunds list
6. ✅ Ledger export
7. ✅ M-Pesa STK Push callback
8. ✅ Failed payment callback
9. ✅ B2C refund callback
10. ✅ Refund endpoint

**Expected output:**
```
========================================
Hive Advanced Features Test Suite
========================================

✅ Dashboard is running
✅ Stats retrieved
✅ Payments listed
✅ Analytics generated
✅ Reconciliation complete
✅ Refunds listed
✅ Ledger export ready
✅ Callback accepted
✅ Failed payment callback handled
✅ B2C callback handled
⚠️  B2C not configured (expected in test)

🎉 All tests passed!
```

---

## Production Deployment

**Final checklist before going live:**

- [ ] M-Pesa production credentials configured
- [ ] Public HTTPS webhook endpoint (Cloudflare/nginx)
- [ ] Admin numbers configured for notifications
- [ ] B2C credentials (if offering refunds)
- [ ] Daily reconciliation scheduled
- [ ] Ledger export tested
- [ ] All tests passing
- [ ] Backups enabled
- [ ] Monitoring alerts set up
- [ ] Bank/lender outreach prepared

**Launch day:**
1. Switch `sandbox: false` in config
2. Restart Hive
3. Place test order with real KES 10
4. Verify webhook receives callback
5. Check admin notification arrives
6. Export ledger and verify data
7. Monitor logs for 24h

---

## Future Enhancements

**Planned features:**
- Customer refund request form (self-service)
- Payment analytics dashboard UI
- Automatic daily reconciliation emails
- Multi-currency support
- Integration with accounting software (QuickBooks, Xero)
- AI-powered fraud detection
- Loan application pre-approval API

**Reality Network DeFi:**
- Collateralized loans using $NET tokens
- On-chain credit scores
- Peer-to-peer lending marketplace
- Automated repayment from revenue

---

## Support & Community

- **Hive GitHub:** https://github.com/kalkiboru111/hive
- **Reality Network:** https://realitynet.xyz
- **M-Pesa Support:** apisupport@safaricom.co.ke
- **Discord:** https://discord.com/invite/clawd

**Got questions? Open an issue!**
