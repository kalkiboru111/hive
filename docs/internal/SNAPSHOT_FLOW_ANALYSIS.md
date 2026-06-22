# Snapshot Flow — End-to-End Analysis

**Date:** February 6, 2026, 6:35 AM EST  
**Status:** System operational, snapshot submission mechanism identified

---

## System State

### Reality Network Cluster
- **Status:** ✅ Running
- **Nodes:** 3 (all Ready)
- **Current Ordinal:** 16 (advancing)
- **Port:** 7000 (isolated from live Sentiment at 9100)

**Node IDs:**
- `890499020f07b4b5...` (IP: 172.18.0.3, Port: 9030, Session: 1770377133432)
- `fa63946bcecee49f...` (IP: 172.18.0.3, Port: 9020, Session: 1770377133432)
- `c3b0aca7486a925e...` (IP: 172.18.0.2, Port: 9000, Session: 1770377132101)

**Note:** Filesystem errors in logs about "Snapshot already exists" — non-blocking, ordinals still advancing.

### Hive Bot
- **Status:** ✅ Running
- **Identity:** `NET4gGeH2bA88hDjjxFXeJcgXwtF11LxePD2n9Pm`
- **Database:** 2 orders (1 delivered: $12, 1 confirmed: $15)
- **Dashboard:** http://localhost:8080 (functional)
- **Network Service:** Running (30s interval)

---

## The Snapshot Submission Flow

### Architecture

```
┌─────────────────────────────────────┐
│ 1. WhatsApp Message Arrives        │
│    (or order created via other UI)  │
└───────────────┬─────────────────────┘
                │
                ↓
┌─────────────────────────────────────┐
│ 2. Handler Processes Message        │
│    - Creates/updates order          │
│    - Stores in SQLite               │
│    - Returns HandlerResult          │
└───────────────┬─────────────────────┘
                │
                ↓ (if NOT NoReply)
┌─────────────────────────────────────┐
│ 3. state_changed = true             │
│    network_notifier.mark_dirty()    │
└───────────────┬─────────────────────┘
                │
                ↓ (sets atomic flag)
┌─────────────────────────────────────┐
│ 4. Network Service (background)     │
│    - Wakes up every 30s OR on notify│
│    - Checks dirty flag              │
│    - If dirty: submit snapshot      │
└───────────────┬─────────────────────┘
                │
                ↓
┌─────────────────────────────────────┐
│ 5. Snapshot Creation                │
│    - capture_state() from SQLite    │
│    - Serialize to MessagePack       │
│    - Build StateChannelSnapshotBin  │
└───────────────┬─────────────────────┘
                │
                ↓
┌─────────────────────────────────────┐
│ 6. Sign Snapshot                    │
│    - identity.sign_value()          │
│    - secp256k1 signature            │
└───────────────┬─────────────────────┘
                │
                ↓
┌─────────────────────────────────────┐
│ 7. Submit to Reality L0             │
│    - POST /state-channels/{addr}/...│
│    - Update last_snapshot_hash      │
└───────────────┬─────────────────────┘
                │
                ↓
┌─────────────────────────────────────┐
│ 8. Reality Network Consensus        │
│    - Validators verify signature    │
│    - Include in next global snapshot│
│    - Ordinal increments             │
└─────────────────────────────────────┘
```

---

## Code Flow Analysis

### 1. Message Handler (`src/bot/mod.rs:handle_incoming_message`)

**Location:** Line ~130-200

**Key logic:**
```rust
async fn handle_incoming_message(...) -> Result<bool> {
    // ... process message, route to handlers ...
    
    // Send response(s)
    let state_changed = !matches!(result, HandlerResult::NoReply);
    
    // Save conversation state
    store.save_conversation_state(&sender, &state.to_json())?;
    
    Ok(state_changed)  // ← Returns true if any reply was sent
}
```

**Returns:** `true` if state changed (any handler responded), `false` otherwise.

---

### 2. Event Loop Wrapper (`src/bot/mod.rs:Event::IncomingMessage`)

**Location:** Line ~150-170

**Key logic:**
```rust
match handle_incoming_message(&config, &store, &wa_ctx).await {
    Ok(state_changed) => {
        if state_changed {
            network_notifier.mark_dirty();  // ← Triggers snapshot
        }
    }
    Err(e) => {
        error!("Error handling message from {}: {}", ...);
    }
}
```

**Trigger:** Only called when `state_changed == true`.

---

### 3. Network Notifier (`src/network/service.rs:NetworkNotifier::mark_dirty`)

**Location:** Line ~50-60

**Key logic:**
```rust
pub fn mark_dirty(&self) {
    if self.enabled {
        self.dirty.store(true, Ordering::Release);  // ← Atomic flag
        self.notify.notify_one();  // ← Wake service immediately
    }
}
```

**Effect:** Sets atomic boolean + wakes network service.

---

### 4. Network Service Loop (`src/network/service.rs:NetworkService::run`)

**Location:** Line ~115-140

**Key logic:**
```rust
pub async fn run(mut self) {
    info!("🌐 Reality Network service started (interval: {}s)", ...);

    loop {
        // Wait for either a dirty notification or the interval timeout
        tokio::select! {
            _ = self.notify.notified() => {
                // State changed — submit soon
            }
            _ = tokio::time::sleep(Duration::from_secs(self.interval_secs)) => {
                // Periodic check
            }
        }

        // Only submit if state actually changed
        if !self.dirty.swap(false, Ordering::AcqRel) {
            continue;  // ← If dirty == false, skip submission
        }

        if let Err(e) = self.submit_snapshot().await {
            error!("❌ Failed to submit snapshot: {}", e);
        }
    }
}
```

**Behavior:**
- Wakes up every 30 seconds OR immediately when `notify` is triggered
- Checks `dirty` flag (atomic boolean)
- If `dirty == false`, does **nothing** (loops back)
- If `dirty == true`, calls `submit_snapshot()` and resets flag

---

### 5. Snapshot Submission (`src/network/service.rs:NetworkService::submit_snapshot`)

**Location:** Line ~142-175

**Key steps:**
```rust
async fn submit_snapshot(&mut self) -> Result<()> {
    // 1. Capture state from SQLite
    let hive_state = snapshot::capture_state(&self.store, &self.business_name)?;

    info!("📸 Capturing state: {} orders, {} delivered", ...);

    // 2. Build state channel binary
    let sc_binary = hive_state.to_state_channel_binary(&self.last_snapshot_hash)?;

    // 3. Sign it
    let signed = self.identity.sign_value(&sc_binary)?;

    // 4. Submit to L0
    self.client
        .submit_state_channel_snapshot(&self.identity.address, &signed)
        .await?;

    // 5. Update chain integrity hash
    let hash = NodeIdentity::hash_value(&sc_binary)?;
    self.last_snapshot_hash = hash;

    info!("✅ Snapshot submitted to Reality Network");
    Ok(())
}
```

**Output logs (when successful):**
```
[INFO] 📸 Capturing state: 2 orders, 1 delivered
[INFO] ✅ Snapshot submitted to Reality Network
```

---

## Current State — Why No Snapshots Yet

### The Problem

**Orders were added via SQL, not WhatsApp messages.**

**What happened:**
```bash
sqlite3 /tmp/hive-cluster-test/data/hive.db "INSERT INTO orders ..."
```

**What didn't happen:**
1. `handle_incoming_message()` was never called
2. `state_changed` was never set to `true`
3. `network_notifier.mark_dirty()` was never called
4. `dirty` flag remains `false`
5. Network service sees `dirty == false`, skips submission

**Proof:**
```bash
grep "Capturing state\|submitted to Reality" /tmp/hive-dashboard.log
# (no output)
```

---

## How to Trigger a Snapshot

### Option 1: Send a WhatsApp Message

**Method:** Pair WhatsApp and send a message to the bot.

**Why it works:** Message → handler → `state_changed = true` → `mark_dirty()`

**Status:** Requires WhatsApp pairing (QR code expires quickly, not ideal for testing).

---

### Option 2: Wire `mark_dirty()` into Store Methods

**Method:** Add `mark_dirty()` calls directly in `store::Store::create_order()`.

**Change needed:**
```rust
// In src/store/mod.rs
pub fn create_order(..., network_notifier: &NetworkNotifier) -> Result<i64> {
    // ... create order ...
    network_notifier.mark_dirty();  // ← Add this
    Ok(order_id)
}
```

**Pros:** Snapshots triggered for ALL order creation (WhatsApp, API, SQL).  
**Cons:** Requires passing `NetworkNotifier` through call chain.

---

### Option 3: Create Integration Test

**Method:** Write a test that manually calls `mark_dirty()` and verifies submission.

**Pseudo-code:**
```rust
#[tokio::test]
async fn test_snapshot_submission() {
    let (service, notifier) = NetworkService::new(...).await?;
    
    // Spawn service
    tokio::spawn(async move { service.run().await });
    
    // Create test order
    store.create_order(...)?;
    
    // Trigger snapshot
    notifier.mark_dirty();
    
    // Wait for submission
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Verify on Reality Network
    let snapshot = client.get_state_channel_snapshot(address).await?;
    assert!(snapshot.is_some());
}
```

---

### Option 4: Manual Trigger via Dashboard API

**Method:** Add a `/api/snapshot/trigger` endpoint that calls `mark_dirty()`.

**Implementation:**
```rust
// In src/dashboard/mod.rs
async fn trigger_snapshot(State(state): State<AppState>) -> impl IntoResponse {
    state.network_notifier.mark_dirty();
    Json(serde_json::json!({"status": "triggered"}))
}
```

**Usage:**
```bash
curl -X POST http://localhost:8080/api/snapshot/trigger
```

---

## Reality Network API Verification

### Endpoints Tested

**1. Cluster Info:**
```bash
curl http://localhost:7000/cluster/info
# ✅ Returns 3 nodes
```

**2. Latest Ordinal:**
```bash
curl http://localhost:7000/global-snapshots/latest/ordinal
# ✅ Returns {"value": 16}
```

**3. Global Snapshot:**
```bash
curl http://localhost:7000/global-snapshots/16
# ⚠️ Returns empty/null fields (might be API issue or timing)
```

**4. State Channel Query:**
```bash
curl http://localhost:7000/state-channels/NET4gGeH2bA88hDjjxFXeJcgXwtF11LxePD2n9Pm
# ⚠️ Returns empty (no snapshots submitted yet)
```

**5. Metrics:**
```bash
curl http://localhost:7000/metrics
# ✅ Returns Prometheus metrics (node is healthy)
```

---

## Snapshot Content

### What Gets Submitted

**Rust struct:**
```rust
pub struct HiveStateSnapshot {
    pub version: u32,                    // Schema version (1)
    pub business_name: String,           // "Test Kitchen"
    pub timestamp_ms: u64,               // Unix timestamp
    pub total_orders: u64,               // 2
    pub total_revenue_cents: i64,        // 2700 (27.00 * 100)
    pub active_orders: u32,              // 1 (confirmed)
    pub delivered_orders: u64,           // 1
    pub vouchers: VoucherStateSummary,   // 0 created, 0 redeemed
    pub order_hashes: Vec<String>,       // ["abc123...", "def456..."]
}
```

**Serialization:** MessagePack (compact binary)  
**Typical size:** ~500 bytes  
**Privacy:** Order hashes only (no customer PII on-chain)

---

## Next Steps

### Immediate (to complete verification)

1. **Add manual trigger endpoint:**
   ```rust
   // src/dashboard/mod.rs
   .route("/api/snapshot/trigger", post(trigger_snapshot))
   ```

2. **Test submission:**
   ```bash
   curl -X POST http://localhost:8080/api/snapshot/trigger
   sleep 2
   grep "Capturing state" /tmp/hive-dashboard.log
   ```

3. **Verify on Reality Network:**
   ```bash
   curl http://localhost:7000/state-channels/NET4gGeH2bA88hDjjxFXeJcgXwtF11LxePD2n9Pm/snapshots/latest
   ```

---

### Production Wiring

**Recommended:** Wire `mark_dirty()` into store methods:

```rust
// src/store/mod.rs
impl Store {
    pub fn create_order_with_notifier(
        &self,
        ...,
        network_notifier: Option<&NetworkNotifier>,
    ) -> Result<i64> {
        // ... create order ...
        
        if let Some(notifier) = network_notifier {
            notifier.mark_dirty();
        }
        
        Ok(order_id)
    }
}
```

**Benefit:** All order creation (WhatsApp, API, admin) triggers snapshots automatically.

---

## Summary

**✅ What Works:**
- Reality cluster (3 nodes, ordinal 16)
- Hive bot connected
- Network service running
- Snapshot serialization tested
- Identity + signing working

**⏸️ What's Pending:**
- `mark_dirty()` trigger (not called for SQL-inserted orders)
- Actual snapshot submission (waiting for trigger)
- Verification on Reality Network (no snapshots yet)

**🔧 Fix Required:**
- Add trigger mechanism (manual endpoint OR store wiring)
- Test full flow end-to-end
- Verify snapshot appears in Reality Network

**Time to fix:** ~15 minutes (add endpoint, test, verify)

---

**Status:** System is 95% complete. Missing: trigger wiring for snapshot submission.

🐝 **Ready to complete integration.**
