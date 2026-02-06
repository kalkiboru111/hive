# 🐝 Hive — WhatsApp Bot Framework for Reality Network

Build and run WhatsApp bots on decentralized infrastructure. No cloud. No monthly fees. Your device, your bot, your business.

## What is Hive?

Hive is a framework that lets anyone create a WhatsApp-based business — ordering systems, customer service, booking, vouchers — and host it on their own device via [Reality Network](https://realitynet.xyz). Zero cloud costs. Works on a laptop or phone.

**✅ Full Reality Network integration validated February 6, 2026** — [See test results](#reality-network-integration)

## Quick Start

### For Builders (Non-Technical)

**👉 [Start here: FOR_BUILDERS.md](FOR_BUILDERS.md)**

Use the interactive wizard:

```bash
./hive wizard my-business
# Answer 4 questions, your config is ready
./hive run my-business/
# Scan QR, bot is live
```

### For Developers

```bash
# Use a template
./hive init --template food-delivery my-bot

# Or start from scratch
./hive init my-bot

# Edit your config
nano my-bot/config.yaml

# Run it
./hive run my-bot/
```

**See all templates:**

```bash
./hive templates
```

## Templates

Hive includes **8 pre-built templates** for common businesses:

- **food-delivery** — Restaurants, street food, home kitchens
- **salon-booking** — Hair salons, barbers, spas
- **event-tickets** — Concerts, workshops, classes
- **tutoring** — Private lessons, test prep
- **voucher-store** — Gift cards, loyalty programs
- **community-store** — Co-ops, farmer's markets
- **customer-support** — Help desk, ticket system
- **real-estate** — Property listings, viewings

**See all templates:**

```bash
./hive templates
```

**Use a template:**

```bash
./hive init --template food-delivery my-restaurant
```

Each template includes pre-filled menu items, messages, and settings — just customize and go.

## Features

- **Config-driven** — define your bot in YAML, no coding required
- **Menu & ordering** — built-in support for product catalogs and order flows
- **Vouchers** — create and redeem voucher codes
- **Admin notifications** — owner gets order alerts via WhatsApp
- **Web dashboard** — manage menu, orders, and analytics from a browser
- **Decentralized hosting** — runs on Reality Network, powered by your community
- **Single binary** — no Docker, no npm, no JVM. Just download and run.

## Architecture

```
Your Device (laptop/phone)
├── WhatsApp Connection (whatsapp-rust)
├── Bot Engine (message routing, conversation state)
├── Plugin System (YAML config + handlers)
├── Web Dashboard (local admin panel)
├── SQLite (sessions, orders, menu)
└── Reality Network Node (rApp integration)
```

## Reality Network Integration

Hive automatically submits state snapshots to Reality Network's L0 layer as a **state channel**. Every order, voucher redemption, and status change is captured and submitted on-chain.

### ✅ Integration Test Results (February 6, 2026)

**Test Cluster:**
- 3-node L0 + 3-node L1 consensus cluster
- Isolated test network (localhost:7000)

**Full End-to-End Flow:**
```
WhatsApp Message → Hive Bot → Order Created → State Changed
→ Snapshot Captured → MessagePack Serialization → L0 Submission
→ Accepted by Cluster → Ordinal Incremented
```

**Logs from Live Test:**
```
[2026-02-06T12:16:30Z INFO hive::bot] 📨 Message from 14152657184@s.whatsapp.net: 3
[2026-02-06T12:16:30Z INFO hive::network::service] 📸 Capturing state: 1 orders, 0 delivered
[2026-02-06T12:16:30Z INFO hive::network::client] Submitting state channel snapshot to http://localhost:7000/state-channels/NET4nFnmFxhdtG9kSR9LXxff35cgTHv6hW8pvzPx/snapshot
[2026-02-06T12:16:30Z INFO hive::network::client] ✅ State channel snapshot accepted by L0
[2026-02-06T12:16:30Z INFO hive::network::service] ✅ Snapshot submitted to Reality Network

[2026-02-06T12:16:51Z INFO hive::handlers::order] 📦 New order #2 from 14152657184@s.whatsapp.net — USD13.00
[2026-02-06T12:16:51Z INFO hive::network::service] 📸 Capturing state: 2 orders, 0 delivered
[2026-02-06T12:16:51Z INFO hive::network::client] ✅ State channel snapshot accepted by L0
[2026-02-06T12:16:51Z INFO hive::network::service] ✅ Snapshot submitted to Reality Network
```

**Network Response:**
- **3 snapshots submitted**
- **All accepted by L0 consensus layer**
- **Ordinal progression:** 12 → 30 (18 snapshots processed)
- **Node identity:** `NET4nFnmFxhdtG9kSR9LXxff35cgTHv6hW8pvzPx`

**Database Verification:**
```bash
$ sqlite3 data/hive.db "SELECT id, customer_phone, total, status FROM orders;"
2|14152657184@s.whatsapp.net|13.0|confirmed
1|+254700111222|27.0|pending
```

### How It Works

1. **Identity Generation:** Hive creates a secp256k1 keypair on first run (`data/identity.json`)
2. **State Capture:** Every message/order triggers snapshot generation
3. **Serialization:** State is encoded using MessagePack
4. **Signing:** Snapshot is cryptographically signed
5. **Submission:** HTTP POST to L0 node `/state-channels/{address}/snapshot`
6. **Consensus:** L0 cluster validates and incorporates into global snapshot
7. **Finality:** Snapshot ordinal increments, state is on-chain

**See:** [docs/REALITY_INTEGRATION.md](docs/REALITY_INTEGRATION.md) for technical details.

## Documentation

- **[For Builders (Non-Technical)](FOR_BUILDERS.md)** — Start here if you're new
- **[Builder's Guide](docs/BUILDERS_GUIDE.md)** — Full walkthrough with examples, tips, FAQ
- **[Video Tutorial](docs/VIDEO_SCRIPT.md)** — 5-minute screencast (production script)
- **[Quickstart](docs/QUICKSTART.md)** — Minimal setup guide
- **[Reality Network Integration](docs/REALITY_INTEGRATION.md)** — Technical deep-dive
- **[Multi-Language Support](docs/MULTI_LANGUAGE.md)** — i18n configuration

## Development

```bash
# Build from source
cargo build --release

# Run tests
cargo test

# Build with Reality Network support (default)
cargo build --release --features network
```

## Roadmap

### Phase 1: Launch (Q1 2026) ✅
- [x] WhatsApp integration (QR pairing)
- [x] Menu & ordering system
- [x] Voucher system
- [x] Web dashboard
- [x] Multi-language support (7 languages)
- [x] Reality Network integration
- [x] MessagePack state serialization
- [x] 8 business templates

**Target:** 10,000 businesses in 6 months  
**Focus:** Africa (Kenya, Nigeria, South Africa)  
**Profile:** Solo entrepreneurs, <100 orders/day  
**Value:** $0/month, 5-min setup, on-chain proof

### Phase 2: SME Features (Q2-Q3 2026)
- [ ] Enhanced analytics & reporting (export CSV, daily summaries)
- [ ] SMS fallback (Twilio integration, pay-per-use)
- [ ] Payment gateway integrations (Stripe, PayStack)
- [ ] M-Pesa support (Kenya)
- [ ] Backup/failover service (Reality nodes offer for $NET)
- [ ] Voice message support
- [ ] Delivery tracking integration
- [ ] Multi-device dashboard improvements

**Target:** Extend "works well" range to 500 orders/day  
**When:** First 100 businesses hit scale constraints

### Phase 3: Business API Bridge (Q4 2026+)
- [ ] WhatsApp Business API support (premium tier)
- [ ] Multi-agent routing (support teams)
- [ ] Template messages (pre-approved broadcasts)
- [ ] Queue management (ticket assignment)
- [ ] Shift handoff (24/7 operations)
- [ ] Enterprise analytics

**Target:** SMEs doing $10k+/month revenue  
**When:** 10+ customers request (and can afford $49/month tier)

### Scaling Considerations

**See [SCALING_ANALYSIS.md](SCALING_ANALYSIS.md)** for detailed breakdown of where current model works (70-80% of SMEs) and where Business API is needed (multi-agent teams, 24/7 operations).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md)

## License

MIT — see [LICENSE](LICENSE)

## Credits

Built on top of:
- [whatsapp-rust](https://github.com/openclaw/whatsapp-rust) — WhatsApp Web protocol
- [Reality Network](https://realitynet.xyz) — Decentralized compute platform

## Support

- Discord: [discord.gg/realitynetwork](https://discord.gg/realitynetwork)
- Twitter: [@RealityNetw0rk](https://twitter.com/RealityNetw0rk)
- Issues: [GitHub Issues](https://github.com/kalkiboru111/hive/issues)

---

**Reality Network Ventures** — First portfolio proof-of-concept  
**Target:** African entrepreneurs, zero cloud costs, 5-minute setup
