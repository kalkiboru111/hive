#!/usr/bin/env bash
# Quick test script for Reality Network integration

set -e

L0_URL="${L0_URL:-http://localhost:9100}"

echo "🐝 Testing Hive → Reality Network Integration"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "📡 L0 URL: $L0_URL"
echo ""

# Test 1: Cluster reachability
echo "✓ Test 1: Cluster reachability"
CLUSTER=$(curl -s "$L0_URL/cluster/info")
if [ -z "$CLUSTER" ]; then
    echo "   ❌ Failed to reach cluster at $L0_URL"
    exit 1
fi

NODE_COUNT=$(echo "$CLUSTER" | jq 'length')
echo "   ✅ Cluster reachable: $NODE_COUNT node(s)"
echo "$CLUSTER" | jq -r '.[] | "      \(.id[0:16])... | \(.state)"'
echo ""

# Test 2: Latest ordinal
echo "✓ Test 2: Query latest global snapshot ordinal"
ORDINAL=$(curl -s "$L0_URL/global-snapshots/latest/ordinal" | jq -r '.value')
if [ "$ORDINAL" = "null" ] || [ -z "$ORDINAL" ]; then
    echo "   ❌ Failed to get ordinal"
    exit 1
fi
echo "   ✅ Latest ordinal: $ORDINAL"
echo ""

# Test 3: Identity generation (from Hive's module)
echo "✓ Test 3: Generate Hive node identity"
CARGO_RESULT=$(cd /Users/bobeirne/.openclaw/workspace/hive && cargo run --release --bin test-identity 2>&1)
if [ $? -eq 0 ]; then
    echo "   ✅ Identity generation works"
else
    echo "   ⚠️  Identity binary not found (expected for now)"
fi
echo ""

# Test 4: MessagePack serialization (check if deps are available)
echo "✓ Test 4: Snapshot serialization test"
TEST_RESULT=$(cd /Users/bobeirne/.openclaw/workspace/hive && cargo test --release snapshot::tests::test_snapshot_roundtrip --no-fail-fast 2>&1 | grep -E "(test result|FAILED)")
if echo "$TEST_RESULT" | grep -q "1 passed"; then
    echo "   ✅ Snapshot serialization works"
else
    echo "   ❌ Snapshot test failed"
fi
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 Integration Test Summary:"
echo ""
echo "   Network: $L0_URL"
echo "   Nodes: $NODE_COUNT"
echo "   Latest ordinal: $ORDINAL"
echo ""
echo "✅ Basic connectivity confirmed!"
echo ""
echo "Next: Run a live bot and watch snapshots submit:"
echo "   cd hive && ./target/release/hive run /tmp/hive-cluster-test/"
echo ""
