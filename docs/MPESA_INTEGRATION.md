# M-Pesa Integration Guide

Complete guide to integrating M-Pesa (Lipa na M-Pesa Online / STK Push) with Hive.

## Overview

M-Pesa is Kenya's mobile money platform (Safaricom). This integration allows customers to pay for orders directly from their phones.

**Flow:**
1. Customer places order and provides delivery location
2. Hive sends STK Push to customer's phone
3. Customer enters M-Pesa PIN to authorize payment
4. Safaricom processes payment and sends confirmation to your webhook
5. Hive confirms order and notifies admin

## Prerequisites

### 1. Get M-Pesa API Credentials

**Sandbox (for testing):**
1. Go to https://developer.safaricom.co.ke
2. Create account and login
3. Navigate to "My Apps" → Create new app
4. Select "Lipa Na M-Pesa Online" API
5. Copy your **Consumer Key** and **Consumer Secret**
6. Get your **Passkey** from the API documentation

**Production:**
1. Contact Safaricom to register as a business (M-Pesa Paybill or Till Number)
2. Apply for Lipa Na M-Pesa Online API access
3. Receive production credentials and shortcode

### 2. Set Up Public Webhook URL

M-Pesa requires a publicly accessible HTTPS endpoint for callbacks.

**Options:**

**A. Deploy with HTTPS (Production)**
```bash
# Example with nginx reverse proxy
server {
    listen 443 ssl;
    server_name yourdomain.com;
    
    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    
    location /api/mpesa/callback {
        proxy_pass http://127.0.0.1:8080;
    }
}
```

**B. Use ngrok (Development)**
```bash
# Start Hive dashboard
hive run /path/to/project

# In another terminal, expose port 8080
ngrok http 8080

# Copy the HTTPS URL (e.g., https://abc123.ngrok.io)
# Your callback URL: https://abc123.ngrok.io/api/mpesa/callback
```

**C. Use Cloudflare Tunnel (Free)**
```bash
# Install cloudflared
brew install cloudflare/cloudflare/cloudflared

# Authenticate
cloudflared tunnel login

# Create tunnel
cloudflared tunnel create hive-mpesa

# Route traffic
cloudflared tunnel route dns hive-mpesa yourdomain.com

# Run tunnel (forwards yourdomain.com → localhost:8080)
cloudflared tunnel run hive-mpesa
```

## Configuration

### 1. Update `config.yaml`

```yaml
business:
  name: "My Business"
  currency: "KES"  # Kenyan Shillings
  phone: "+254722000000"

payments:
  enabled: true
  mpesa:
    consumer_key: "YOUR_CONSUMER_KEY"
    consumer_secret: "YOUR_CONSUMER_SECRET"
    shortcode: "174379"  # Your business shortcode
    passkey: "YOUR_PASSKEY"
    callback_url: "https://yourdomain.com/api/mpesa/callback"
    sandbox: true  # false for production

dashboard:
  enabled: true
  port: 8080
```

### 2. Test Configuration

Start the bot:
```bash
hive run /path/to/project
```

You should see:
```
💰 M-Pesa payments enabled (sandbox)
🌐 Dashboard running at http://localhost:8080
```

## Testing

### 1. Sandbox Test Numbers

Safaricom provides test phone numbers for sandbox:
- **254708374149** - Returns success
- **254700000000** - Returns user cancelled
- **254711111111** - Returns insufficient funds

### 2. Place Test Order

1. Send WhatsApp message to your bot: `1` (view menu)
2. Select item: `1`
3. Confirm order: `YES`
4. Send location: `Test Location`
5. Check your phone for STK Push prompt
6. Enter test PIN (sandbox PIN is `1234`)

### 3. Verify Webhook

Check logs for callback:
```
📥 M-Pesa callback received: CheckoutRequestID=ws_CO_..., ResultCode=0
✅ M-Pesa payment successful: Receipt=NLJ7RT61SV, Amount=100.0, Phone=254708374149
💰 Payment PAY-123-1234567890 completed — Order #123 confirmed
```

### 4. Test Webhook Manually

```bash
# Simulate successful payment callback
curl -X POST http://localhost:8080/api/mpesa/callback \
  -H "Content-Type: application/json" \
  -d '{
    "Body": {
      "stkCallback": {
        "MerchantRequestID": "29115-34620561-1",
        "CheckoutRequestID": "ws_CO_191220191020363925",
        "ResultCode": 0,
        "ResultDesc": "The service request is processed successfully.",
        "CallbackMetadata": {
          "Item": [
            {"Name": "Amount", "Value": 100.00},
            {"Name": "MpesaReceiptNumber", "Value": "TEST123"},
            {"Name": "TransactionDate", "Value": 20240206120000},
            {"Name": "PhoneNumber", "Value": 254722000000}
          ]
        }
      }
    }
  }'
```

Expected response:
```json
{
  "ResultCode": 0,
  "ResultDesc": "Accepted"
}
```

## Production Checklist

- [ ] Switch to production credentials (`sandbox: false`)
- [ ] Set up HTTPS endpoint (Let's Encrypt, Cloudflare, etc.)
- [ ] Update `callback_url` to production domain
- [ ] Test with real money (start with small amounts)
- [ ] Monitor webhook logs for failures
- [ ] Set up alerts for failed payments
- [ ] Implement retry logic for network failures
- [ ] Add admin notification for successful payments
- [ ] Comply with Safaricom API rate limits
- [ ] Store M-Pesa receipt numbers for reconciliation

## Troubleshooting

### STK Push Not Received

1. **Check phone number format**
   - Must be in format `254722000000` (no +, spaces, or dashes)
   - Hive auto-formats, but verify in logs

2. **Check shortcode and passkey**
   - Sandbox: Use test shortcode from Daraja portal
   - Production: Use your registered business shortcode

3. **Check API credentials**
   - Consumer key/secret must match your app
   - Regenerate if expired (tokens expire every hour)

### Webhook Not Working

1. **Check callback URL**
   ```bash
   curl https://yourdomain.com/api/mpesa/callback
   # Should return 405 Method Not Allowed (POST only)
   ```

2. **Check logs**
   ```bash
   tail -f /path/to/project/hive.log
   ```

3. **Verify HTTPS**
   - M-Pesa only calls HTTPS endpoints
   - Self-signed certs won't work in production

4. **Check firewall**
   - Safaricom IPs must reach your server
   - Whitelist: `196.201.214.0/24` (Safaricom range)

### Payment Stuck in "Processing"

1. **Callback not received**
   - Check webhook logs
   - Verify callback URL is correct and accessible

2. **Customer didn't complete payment**
   - STK Push expires after 60 seconds
   - Customer may have cancelled or entered wrong PIN

3. **Network timeout**
   - Safaricom may retry callback (implement idempotency)
   - Check for duplicate `CheckoutRequestID` in logs

## Security Notes

1. **Validate callbacks**
   - Future: Implement signature verification
   - Check `CheckoutRequestID` exists in your database

2. **Idempotency**
   - M-Pesa may send duplicate callbacks
   - Check payment status before updating

3. **Rate limiting**
   - Safaricom API has rate limits (check your tier)
   - Implement request queuing for high volume

4. **Credentials**
   - Never commit credentials to git
   - Use environment variables or secrets manager
   - Rotate keys periodically

## Cost

**Safaricom Fees (2024):**
- **Paybill:** 0% for under KES 2,500 transactions (May vary)
- **Till Number:** Flat fee per transaction
- **API Access:** Free (after registration)

Check current rates: https://www.safaricom.co.ke/business/payments/m-pesa-rates

## Support

- **Safaricom Developer Portal:** https://developer.safaricom.co.ke
- **API Documentation:** https://developer.safaricom.co.ke/APIs/LipaNaMPesaOnline
- **Support Email:** apisupport@safaricom.co.ke
- **Hive Issues:** https://github.com/kalkiboru111/hive/issues

## Next Steps

- Implement admin WhatsApp notifications for completed payments
- Add retry logic for failed webhooks
- Build payment reconciliation dashboard
- Support M-Pesa B2C (payouts to customers)
- Add payment analytics and reporting
