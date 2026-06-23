#!/bin/bash
# Wrapper: build + sign the Hive rApp deploy tx, then POST it.
# Configure via the HIVE_* env vars (see README.md). Run from the deploy/ dir.
set -euo pipefail
cd "$(dirname "$0")"

OUT="${1:-/tmp/hive-deploy.tx}"
TX_URL="${HIVE_L0_TX_URL:-http://143.110.227.9:9100/transactions}"

if [ ! -f lib/reality-combined.jar ]; then
  echo "❌ Missing lib/reality-combined.jar — see README.md (drop the Reality assembly jar there)." >&2
  exit 1
fi

echo "🐝 Building + signing deploy transaction → $OUT"
sbt -batch "runMain CreateHiveDeployAppTx $OUT"

echo "📤 Posting to $TX_URL"
curl -fsS -X POST "$TX_URL" -H 'Content-Type: application/json' --data @"$OUT"
echo

echo "✅ Posted. Next: export your key (keytool export) and set network.identity_key_hex"
echo "   in your bot's config.yaml, then run ./hive run <project>."
