# Hive Quickstart

## Prerequisites
- A phone with WhatsApp
- A second phone (or WhatsApp Web) to test as a customer

## Setup (2 minutes)

```bash
# 1. Download binary (choose your platform)
# Linux:
curl -LO https://github.com/kalkiboru111/hive/releases/latest/download/hive-linux-x86_64
mv hive-linux-x86_64 hive

# macOS:
curl -LO https://github.com/kalkiboru111/hive/releases/latest/download/hive-macos-arm64
mv hive-macos-arm64 hive

# 2. Make executable
chmod +x hive

# 3. Create minimal config
cat > config.yaml << 'EOF'
business:
  name: "Test Kitchen"
  currency: "USD"
  welcome: "Welcome! Reply 1 for menu, 2 for orders"

menu:
  - name: "Burger"
    price: 10.00
    emoji: "🍔"

admin_numbers:
  - "+YOUR_NUMBER_HERE"

# Reality is off until you deploy your rApp — see docs/DEPLOY.md, then set enabled: true
network:
  enabled: false
  l0_url: "http://143.110.227.9:9000"
EOF

# 4. Run
./hive run .
```

## Test Checklist

- [ ] QR code appears
- [ ] WhatsApp pairs successfully
- [ ] Bot responds to "1" with menu
- [ ] Can place order ("order 1")
- [ ] Admin receives notification
- [ ] "DONE 1" marks order complete
- [ ] Dashboard shows at localhost:8080
- [ ] (After deploying your rApp — see docs/DEPLOY.md) logs show "Snapshot submitted to Reality Network"

## Report Issues

https://github.com/kalkiboru111/hive/issues
