# 🌐 Reality Network Integration

How Hive connects to Reality Network's L0/L1 consensus layer.

---

## Architecture

```
┌─────────────────┐
│ Hive Bot        │
│ (Rust/WhatsApp) │
└────────┬────────┘
         │
         │ State Channel Snapshots
         │ (MessagePack binary)
         ↓
┌─────────────────┐
│ Reality L0 Node │
│ (REST API)      │
└────────┬────────┘
         │
         │ Global Consensus
         ↓
┌─────────────────┐
│ DAG Network     │
│ (Distributed)   │
└─────────────────┘
```

---

## What Gets Submitted

Hive submits **state channel snapshots** containing:

```rust
pub struct HiveStateSnapshot {
    pub version: u32,                    // Schema version
    pub business_name: String,           // Business identifier
    pub timestamp_ms: u64,               // When snapshot was taken
    pub total_orders: u64,               // Lifetime order count
    pub total_revenue_cents: i64,        // Lifetime revenue (smallest unit)
    pub active_orders: u32,              // Currently pending orders
    pub delivered_orders: u64,           // Completed orders
    pub vouchers: VoucherStateSummary,   // Voucher state
    pub order_hashes: Vec<String>,       // Hashed order IDs (no PII)
}
```

**Privacy:** Customer data (names, phone numbers, addresses) NEVER goes on-chain.  
**Proof:** Order hashes prove orders exist without exposing PII.

---

## How It Works

### 1. Identity Generation

First run generates a Reality Network identity:

```bash
./hive run my-bot/
```

Creates `data/identity.json`:
```json
{
  "address": "NET1hrf37iZr564XaGj3WYmVA6ko2ipNiPE5s8U6",
  "peer_id_hex": "7ec384c6756103a9...",
  "private_key_hex": "..." 
}
```

This identity is your bot's address on Reality Network.

---

### 2. Cluster Connection

On startup, Hive checks if Reality cluster is reachable:

```
[INFO] 🔗 Reality Network identity: NET1hrf37iZr564XaGj3WYmVA6ko2ipNiPE5s8U6
[INFO] Reality cluster: 1 nodes
[INFO] ✅ Reality cluster reachable: 1 node(s)
[INFO] 🌐 Reality Network service started (interval: 30s)
```

---

### 3. Snapshot Submission

**Triggers:**
- Every 30 seconds (configurable)
- After state changes (orders, vouchers)

**Process:**
1. Bot captures current state from SQLite
2. Serializes to MessagePack bytes
3. Signs with identity private key
4. Submits via `POST /state-channels/{address}/snapshot`

**Logs:**
```
[INFO] 📸 Snapshot captured: 5 orders, $125.00 revenue
[INFO] ✅ Snapshot submitted to Reality Network (ordinal: 6064)
```

---

## Configuration

Edit `config.yaml`:

```yaml
network:
  enabled: true                         # Enable Reality Network sync
  l0_url: "http://localhost:9100"       # L0 node endpoint
  identity_path: "data/identity.json"   # Where to store identity
  snapshot_interval_secs: 30            # How often to submit
```

**Common endpoints:**
- **Local cluster:** `http://localhost:7100` (port 7100 to avoid conflict with live nodes)
- **Live Sentiment testnet:** `http://localhost:9100` (Mac Mini Tailscale, DO NOT USE for testing)
- **Mainnet:** (TBD)

⚠️ **CRITICAL:** The Mac Mini runs Sentiment at `100.123.52.97:9100` (Tailscale). This resolves to `localhost:9100` locally. Always use port `7100` for local testing to avoid interfering with live rApps.

---

## Testing with Local Cluster

### 1. Start Reality Cluster

```bash
cd /Users/bobeirne/Downloads/reality-main
docker-compose -f docker-compose.dev.yml up -d
```

**Verify:**
```bash
curl http://localhost:9100/cluster/info | jq
```

Expected output:
```json
[
  {
    "id": "642503c6b909ffe3...",
    "ip": "127.0.0.1",
    "publicPort": 9100,
    "state": "Ready"
  }
]
```

---

### 2. Configure Hive Bot

```yaml
network:
  enabled: true
  l0_url: "http://localhost:9100"
```

---

### 3. Run Bot

```bash
./hive run my-bot/
```

**Watch for:**
```
✅ Reality cluster reachable: 1 node(s)
🌐 Reality Network service started
```

---

### 4. Create Activity

Place orders (via WhatsApp or manually in SQLite):

```sql
INSERT INTO orders (customer_jid, customer_phone, total_amount, status) 
VALUES ('test@s.whatsapp.net', '+1234567890', 35.00, 'confirmed');
```

---

### 5. Verify Snapshot Submission

**Watch bot logs:**
```bash
tail -f my-bot/logs/hive.log | grep snapshot
```

**Query Reality Network:**
```bash
curl http://localhost:9100/state-channels/NET1hrf37iZr564XaGj3WYmVA6ko2ipNiPE5s8U6/snapshots/latest
```

---

## API Endpoints (Reality L0 Node)

### Cluster Info
```bash
GET /cluster/info
```

Returns list of nodes in the cluster.

---

### Latest Ordinal
```bash
GET /global-snapshots/latest/ordinal
```

Returns the current global snapshot ordinal (block height equivalent).

---

### Submit Snapshot
```bash
POST /state-channels/{address}/snapshot
Content-Type: application/json

{
  "lastSnapshotHash": "abc123...",
  "content": "base64-encoded-messagepack-bytes",
  "signature": "..."
}
```

---

### Query State Channel
```bash
GET /state-channels/{address}/snapshots/latest
```

Returns the most recent snapshot submitted by this bot.

---

## State Channel Model

Each Hive bot = a state channel on Reality Network.

**Benefits:**
- **Sovereign:** Bot controls its own state
- **Verifiable:** Anyone can audit snapshot history
- **Decentralized:** No central database
- **Censorship-resistant:** Can't be shut down
- **Franchisable:** Others can run copies, original owner earns fees

**Future:**
- State channels can interact (cross-bot orders, shared vouchers)
- Multi-sig ownership (co-owned businesses)
- Governance via snapshot voting

---

## Troubleshooting

### ❌ "Reality cluster not reachable"

**Cause:** L0 node not running or wrong URL.

**Fix:**
1. Check Docker: `docker ps | grep reality`
2. Test endpoint: `curl http://localhost:9100/cluster/info`
3. Verify config: `l0_url` matches running node

---

### ❌ "Failed to submit snapshot"

**Cause:** Invalid signature or network issue.

**Fix:**
1. Check identity file exists: `cat data/identity.json`
2. Regenerate if corrupted: `rm data/identity.json && restart bot`
3. Check L0 logs: `docker logs reality-main-combined-1`

---

### ❌ "Snapshot validation failed"

**Cause:** Schema version mismatch or corrupted MessagePack.

**Fix:**
1. Update Hive to latest version
2. Check Reality Network protocol version matches
3. Reset state: `rm data/hive.db && restart`

---

## Performance

**Snapshot size:** ~500 bytes (compressed MessagePack)  
**Submission frequency:** Every 30 seconds (default)  
**Bandwidth:** <1 KB/min per bot  
**Latency:** <100ms (local cluster), <500ms (testnet)

**Scaling:**
- 1 bot = 1 state channel
- 1 L0 node can handle 1000s of bots
- No centralized database bottleneck

---

## Security

**Private key storage:** Local filesystem only (`data/identity.json`)  
**Encryption:** secp256k1 signatures  
**PII protection:** Customer data never on-chain  
**Replay protection:** Each snapshot references previous hash  
**Audit trail:** Full snapshot history queryable on-chain

---

## Next Steps

**Planned features:**
- [ ] State channel → rApp deployment (one-click publish)
- [ ] Cross-bot interactions (shared vouchers, group orders)
- [ ] Snapshot pruning (archive old snapshots off-chain)
- [ ] Multi-sig ownership (co-op businesses)
- [ ] Governance (vote on config changes via snapshots)
- [ ] Payment rails (NET, stablecoins, fiat on-ramps)

---

## Further Reading

- Reality Network SDK: `reality-sdk-textbook.txt` (workspace)
- State Channel Docs: `memory/2026-02-02-reality-sdk.md`
- Protocol Source: `/Users/bobeirne/Downloads/reality-main/`

---

🐝 **Hive + Reality Network = Decentralized business infrastructure.**
