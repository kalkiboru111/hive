#!/bin/bash
# Hive rApp Setup Script
# Sets up Hive WhatsApp bot connected to Reality Network testnet

set -e

# Testnet endpoints - update these with your testnet's public address
TESTNET_L0="${HIVE_TESTNET_L0:-http://100.123.52.97:9100}"
TESTNET_L1="${HIVE_TESTNET_L1:-http://100.123.52.97:9110}"
RELEASES_URL="https://github.com/kalkiboru111/hive/releases/latest/download"

echo "🐝 Hive rApp Setup"
echo "=================="
echo ""

# Detect platform
detect_platform() {
    local os=$(uname -s)
    local arch=$(uname -m)
    
    if [[ "$os" == "Linux" && "$arch" == "x86_64" ]]; then
        echo "linux-x86_64"
    elif [[ "$os" == "Linux" && "$arch" == "aarch64" ]]; then
        echo "linux-arm64"
    elif [[ "$os" == "Darwin" && "$arch" == "arm64" ]]; then
        echo "macos-arm64"
    elif [[ "$os" == "Darwin" && "$arch" == "x86_64" ]]; then
        echo "macos-x86_64"
    else
        echo "unknown"
    fi
}

PLATFORM=$(detect_platform)
echo "Detected platform: $PLATFORM"

if [[ "$PLATFORM" == "unknown" ]]; then
    echo "❌ Unsupported platform. Please build from source."
    exit 1
fi

# Create directory
HIVE_DIR="${HIVE_DIR:-$HOME/hive}"
mkdir -p "$HIVE_DIR"
cd "$HIVE_DIR"

echo "Installing to: $HIVE_DIR"
echo ""

# Download binary if not present
if [[ ! -f "hive" ]]; then
    echo "📥 Downloading Hive binary..."
    BINARY_NAME="hive-$PLATFORM"
    
    if command -v curl &> /dev/null; then
        curl -fsSL "$RELEASES_URL/$BINARY_NAME" -o hive || {
            echo "❌ Download failed. You may need to download manually from:"
            echo "   https://github.com/kalkiboru111/hive/releases"
            exit 1
        }
    elif command -v wget &> /dev/null; then
        wget -q "$RELEASES_URL/$BINARY_NAME" -O hive || {
            echo "❌ Download failed."
            exit 1
        }
    else
        echo "❌ Please install curl or wget"
        exit 1
    fi
    
    chmod +x hive
    echo "✅ Binary downloaded"
else
    echo "✅ Binary already present"
fi

# Create config if not present
if [[ ! -f "config.yaml" ]]; then
    echo ""
    echo "📝 Creating config.yaml..."
    echo ""
    
    read -p "Business name: " BUSINESS_NAME
    BUSINESS_NAME="${BUSINESS_NAME:-My Business}"
    
    read -p "Currency (e.g., USD, ZAR, EUR): " CURRENCY
    CURRENCY="${CURRENCY:-USD}"
    
    read -p "Your WhatsApp number for admin alerts (e.g., +1234567890): " ADMIN_NUMBER
    
    cat > config.yaml << EOF
# Hive Configuration
# Edit this file to customize your bot

business:
  name: "$BUSINESS_NAME"
  currency: "$CURRENCY"
  welcome: |
    Welcome to $BUSINESS_NAME! 🐝
    
    Reply with a number:
    1. 📋 View Menu
    2. 📦 My Orders
    3. 🎟️ Redeem Voucher
    4. ℹ️ About Us
  about: "Powered by Hive on Reality Network"

menu:
  - name: "Item 1"
    price: 10.00
    emoji: "🍕"
    description: "Your first menu item"
  - name: "Item 2"
    price: 15.00
    emoji: "🍔"
    description: "Your second menu item"

delivery:
  fee: 5.00
  estimate_minutes: [30, 45]

admin_numbers:
  - "$ADMIN_NUMBER"

# Reality Network connection
network:
  enabled: true
  l0_url: "$TESTNET_L0"
  identity_path: "data/identity.json"
  snapshot_interval_secs: 30

dashboard:
  port: 8080
  enabled: true
EOF

    echo "✅ Config created"
    echo ""
    echo "📌 Edit config.yaml to customize your menu and settings"
else
    echo "✅ Config already present"
fi

# Create data directory
mkdir -p data

echo ""
echo "=========================================="
echo "✅ Setup complete!"
echo "=========================================="
echo ""
echo "Next steps:"
echo ""
echo "1. Edit config.yaml to customize your menu"
echo ""
echo "2. Run Hive:"
echo "   cd $HIVE_DIR"
echo "   ./hive"
echo ""
echo "3. Scan the QR code with WhatsApp to pair"
echo ""
echo "4. Test by sending a message to your WhatsApp number"
echo ""
echo "Testnet endpoints:"
echo "  L0: $TESTNET_L0"
echo "  L1: $TESTNET_L1"
echo ""
echo "Dashboard: http://localhost:8080 (when running)"
echo ""
