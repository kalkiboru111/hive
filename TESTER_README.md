# Hive rApp Tester Kit

**Hive** is a WhatsApp bot framework that runs as an rApp on Reality Network. Each Hive instance is a business (food delivery, services, etc.) that stores order state on the Reality Network.

## Quick Start

### Option 1: Automated Setup

```bash
curl -fsSL https://raw.githubusercontent.com/kalkiboru111/hive/main/setup.sh | bash
```

### Option 2: Manual Setup

1. **Download the binary** for your platform:
   - [Linux x86_64](https://github.com/kalkiboru111/hive/releases/latest/download/hive-linux-x86_64)
   - [macOS ARM64](https://github.com/kalkiboru111/hive/releases/latest/download/hive-macos-arm64)

2. **Make it executable:**
   ```bash
   chmod +x hive
   ```

3. **Create config.yaml** (see [Configuration](#configuration) below)

4. **Run Hive (from project directory):**
   ```bash
   ./hive run .
   ```

5. **Scan the QR code** with WhatsApp (Link a Device)

---

## Configuration

Create a `config.yaml` file:

```yaml
business:
  name: "My Food Delivery"
  currency: "USD"
  welcome: |
    Welcome! 🐝
    
    Reply with a number:
    1. 📋 View Menu
    2. 📦 My Orders
    3. 🎟️ Redeem Voucher
    4. ℹ️ About Us
  about: "Fast delivery, great food!"

menu:
  - name: "Burger"
    price: 12.00
    emoji: "🍔"
    description: "Classic beef burger"
  - name: "Pizza"
    price: 15.00
    emoji: "🍕"
    description: "Margherita pizza"
  - name: "Salad"
    price: 8.00
    emoji: "🥗"
    description: "Fresh garden salad"

delivery:
  fee: 3.00
  estimate_minutes: [20, 35]

admin_numbers:
  - "+1234567890"  # Your number for order alerts

network:
  enabled: true
  l0_url: "http://100.123.52.97:9100"
  identity_path: "data/identity.json"
  snapshot_interval_secs: 30

dashboard:
  port: 8080
  enabled: true
```

---

## Testing Flow

### 1. Start Hive
```bash
./hive run .
```

You'll see:
```
🐝 Hive starting...
📱 Scan QR code to pair WhatsApp
[QR CODE APPEARS]
```

### 2. Pair WhatsApp
- Open WhatsApp on your phone
- Go to Settings → Linked Devices → Link a Device
- Scan the QR code

### 3. Test the Bot
Send a message to your WhatsApp number from another phone:
- Send "1" to see the menu
- Send "order 1" to order item #1
- Send your location when prompted
- Admin receives notification

### 4. Admin Commands
From your admin number:
- `ADMIN` — Enter admin mode
- `DONE <order_id>` — Mark order delivered
- `VOUCHER <amount>` — Create voucher
- `EXIT` — Exit admin mode

### 5. Check Dashboard
Open http://localhost:8080 to see:
- Order history
- Revenue stats
- Active orders

---

## Reality Network Integration

Hive submits state snapshots to the Reality Network testnet:

| Endpoint | URL |
|----------|-----|
| L0 (Global) | http://100.123.52.97:9100 |
| L1 (State Channels) | http://100.123.52.97:9110 |

### What gets stored on-chain:
- Order hashes (not customer PII)
- Voucher issuance/redemption
- Business state snapshots

### Identity
On first run, Hive generates a secp256k1 keypair at `data/identity.json`. This identity signs all state channel submissions.

---

## Troubleshooting

### QR code not showing
- Make sure no other WhatsApp Web session is active
- Try deleting `data/session` folder and restart

### Can't connect to testnet
- Check if testnet is up: `curl http://100.123.52.97:9100/cluster/info`
- Testnet may be restarting — wait a minute and retry

### Orders not persisting
- Check `data/hive.db` exists (SQLite database)
- Check logs for Reality Network submission errors

---

## File Structure

```
hive/
├── hive              # Binary
├── config.yaml       # Your configuration
└── data/
    ├── identity.json # Your Reality Network identity
    ├── hive.db       # Local SQLite database
    └── session/      # WhatsApp session data
```

---

## Support

- **GitHub Issues:** https://github.com/kalkiboru111/hive/issues
- **Telegram:** https://t.me/realitynetw0rk
- **Discord:** (coming soon)

---

## About Reality Network

Reality Network is a decentralized compute platform where your community is your infrastructure. Hive is one of the first rApps demonstrating how businesses can run on the network.

Learn more: https://realitynet.xyz
