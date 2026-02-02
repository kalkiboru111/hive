# 🐝 Hive — WhatsApp Bot Framework for Reality Network

Build and run WhatsApp bots on decentralized infrastructure. No cloud. No monthly fees. Your device, your bot, your business.

## What is Hive?

Hive is a framework that lets anyone create a WhatsApp-based business — ordering systems, customer service, booking, vouchers — and host it on their own device via [Reality Network](https://realitynet.xyz). Zero cloud costs. Works on a laptop or phone.

**Status: Live and tested.** The bot is running on WhatsApp, processing orders, and submitting state channel snapshots to Reality Network's L0 consensus layer.

## Quick Start

```bash
# Download the binary (or build from source)
./hive init my-bot

# Edit your config
nano my-bot/config.yaml

# Run it — scan the QR code with WhatsApp
./hive run my-bot/
```

That's it. Your bot is live.

## What's Working

Everything below has been built, tested, and confirmed working:

### WhatsApp Bot Engine ✅
- QR code pairing (scan with WhatsApp to connect)
- Full conversation state machine (idle → menu → item selection → cart → confirm → location → order placed)
- Message routing with handler chain
- Admin mode toggle (`ADMIN`/`EXIT`) for same-number testing
- Admin notifications on new orders
- Customer delivery notifications (`DONE <id>`)
- Persistent sessions via SQLite
- Auto-reconnect via whatsapp-rust keepalive

### Ordering System ✅
- Config-driven YAML menus with prices, descriptions, emoji
- Multi-item cart with quantity parsing ("2x Kota", "3 Bunny Chow")
- Order confirmation flow with delivery estimates
- Location capture (text address or WhatsApp location pin)
- Order status tracking (pending → confirmed → delivered)

### Voucher System ✅
- Admin voucher creation with custom amounts
- Short code generation (no ambiguous characters)
- Voucher redemption with balance validation
- Redemption tracking in SQLite

### Reality Network Integration ✅
- **State channel snapshots accepted by L0 consensus** — confirmed on-chain
- secp256k1 identity generation with proper Reality address derivation (SHA256 → Base58 → parity prefix)
- SHA512withECDSA signing protocol matching Reality's JVM implementation
- MessagePack serialization for compact on-chain state
- Signed byte encoding matching Java/Circe `Array[Byte]` format
- Background `NetworkService` submits snapshots on state changes (rate-limited)
- Node identity persisted to `data/identity.json`
- Tested against 2-node local Reality cluster — snapshots included in global snapshots

### Infrastructure ✅
- Single 7.3MB release binary (Rust, LTO optimized)
- No JVM, no Docker, no npm — just the binary + config.yaml
- SQLite for all local storage
- Web dashboard (Axum) on configurable port
- CLI: `hive init`, `hive run`, `hive run --phone <number>`, `hive dashboard`
- 28 unit tests passing

## Example: Cloudy Deliveries

A food delivery bot for South African townships — the first Hive template.

```yaml
business:
  name: "Cloudy Deliveries"
  currency: "ZAR"
  welcome: "Welcome to Cloudy Deliveries! 🍔☁️"

menu:
  - name: "Kota"
    price: 35.00
    emoji: "🌯"
    description: "Classic township kota - chips, polony, atchar, cheese"
  - name: "Bunny Chow"
    price: 45.00
    emoji: "🍛"
    description: "Half loaf filled with curry"
  - name: "Pap & Vleis"
    price: 50.00
    emoji: "🍖"

# Reality Network integration
network:
  enabled: true
  l0_url: "http://localhost:7000"
  snapshot_interval_secs: 30
```

## Architecture

```
Your Device (laptop/phone)
├── WhatsApp Connection (whatsapp-rust, WebSocket)
├── Bot Engine (conversation state machine, message routing)
├── Handlers (menu, ordering, vouchers, admin)
├── SQLite Store (orders, sessions, vouchers)
├── Network Service (background snapshot submission)
│   ├── Identity (secp256k1 keypair, Reality address)
│   ├── Snapshot (MessagePack serialization)
│   └── Client (HTTP → Reality L0 node)
└── Web Dashboard (Axum, port 8080)
```

### On-Chain Data Flow

```
WhatsApp message
  → Bot processes order/voucher
  → State captured from SQLite
  → Serialized to MessagePack
  → Wrapped in StateChannelSnapshotBinary
  → Signed with SHA512withECDSA (secp256k1)
  → POST /state-channels/{address}/snapshot
  → L0 consensus accepts
  → Included in next global snapshot
```

Order hashes go on-chain, not customer PII. Each Hive business instance = a state channel address on Reality Network.

## Building from Source

```bash
git clone https://github.com/kalkiboru111/hive
cd hive
cargo build --release
# Binary at target/release/hive (~7.3MB)
```

### Running the Integration Test

```bash
# Requires a Reality L0 node at localhost:7000
cargo run --example test_reality
```

## Config Reference

| Section | Key | Description |
|---------|-----|-------------|
| `business` | `name`, `currency`, `welcome`, `about` | Business identity |
| `menu[]` | `name`, `price`, `emoji`, `description`, `available` | Menu items |
| `delivery` | `fee`, `estimate_minutes`, `radius_km` | Delivery settings |
| `admin_numbers[]` | Phone numbers | Admin WhatsApp numbers |
| `messages` | `order_confirmed`, `order_delivered`, etc. | Customizable templates |
| `dashboard` | `port`, `enabled` | Web dashboard |
| `network` | `enabled`, `l0_url`, `identity_path`, `snapshot_interval_secs` | Reality Network |

## What's Next

- [ ] Register Hive as an rApp via `DeployAppTransaction`
- [ ] Payment integration (SnapScan, Zapper, mobile money)
- [ ] Multi-language support (Zulu, Xhosa, Afrikaans)
- [ ] Auto-reconnect on WebSocket drops
- [ ] Persistent deployment (launchd/systemd service)
- [ ] Dashboard UI improvements

## License

MIT

---

Built by [Reality Network](https://realitynet.xyz) — your community is your infrastructure.
