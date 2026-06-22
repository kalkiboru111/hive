#!/bin/bash
# Deploy a Hive rApp to the Reality testnet.
#
# Hive does NOT deploy itself — this wraps Reality's `wallet` CLI to create and
# post a createDeployAppTransaction, then reminds you to point Hive at the same
# key. See docs/DEPLOY.md for the full explanation and how to obtain the jars.
#
# Confirm flag spellings with `java -jar "$WALLET_JAR" create-deploy-app-transaction --help`.
set -euo pipefail

# ── Configure these (or pass as environment variables) ──────────────────────
WALLET_JAR="${WALLET_JAR:?set WALLET_JAR to the reality wallet jar}"
KEYSTORE="${KEYSTORE:?set KEYSTORE to your rApp key .p12 (see keytool generate)}"
KEY_ALIAS="${KEY_ALIAS:?set KEY_ALIAS}"
KEY_PASSWORD="${KEY_PASSWORD:?set KEY_PASSWORD}"

L0_URL="${L0_URL:-http://143.110.227.9:9000}"        # Reality testnet genesis L0
DESTINATION="${DESTINATION:?set DESTINATION to the deploy NET address (ask your operator)}"

APP_BINARY="${APP_BINARY:?set APP_BINARY to the local Hive binary (its SHA-256 is recorded)}"
APP_NAME="${APP_NAME:-my-hive-bot}"
APP_VERSION="${APP_VERSION:-0.2.0}"
APP_DESCRIPTION="${APP_DESCRIPTION:-Hive WhatsApp commerce bot}"
APP_DOWNLOAD_URL="${APP_DOWNLOAD_URL:?set APP_DOWNLOAD_URL to the published binary URL}"
TOKEN_TICKER="${TOKEN_TICKER:-MYBOT}"
TOTAL_SUPPLY="${TOTAL_SUPPLY:-1000000}"
TOKEN_PRICE="${TOKEN_PRICE:-1}"
TOKENS_FOR_SALE="${TOKENS_FOR_SALE:-0}"

OUT_TX="${OUT_TX:-./deploy-tx.json}"
# ────────────────────────────────────────────────────────────────────────────

echo "🐝 Creating deploy transaction (free on testnet: fee/amount default 0)..."
java -jar "$WALLET_JAR" create-deploy-app-transaction \
  --keystore "$KEYSTORE" --alias "$KEY_ALIAS" --password "$KEY_PASSWORD" \
  --destination "$DESTINATION" \
  --appDataPath "$APP_BINARY" \
  --appName "$APP_NAME" \
  --appVersion "$APP_VERSION" \
  --appDescription "$APP_DESCRIPTION" \
  --appDownloadURL "$APP_DOWNLOAD_URL" \
  --tokenTicker "$TOKEN_TICKER" \
  --totalSupply "$TOTAL_SUPPLY" \
  --tokenPrice "$TOKEN_PRICE" \
  --tokensForSale "$TOKENS_FOR_SALE" \
  --nextTxPath "$OUT_TX"

echo "📤 Posting to $L0_URL/transactions ..."
curl -fsS -X POST "$L0_URL/transactions" \
  -H 'Content-Type: application/json' \
  --data @"$OUT_TX"
echo

echo "✅ Deploy transaction posted."
echo "Next: export your key (keytool export) and set network.identity_key_hex in config.yaml,"
echo "then set network.enabled: true and run ./hive run <project>/  (see docs/DEPLOY.md)."
