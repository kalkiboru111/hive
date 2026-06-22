# 🐝 Hive — WhatsApp Bot Framework for Reality Network

Build and run WhatsApp commerce bots — ordering, bookings, vouchers, payments — and host
them on the decentralized [Reality Network](https://github.com/reality-foundation) instead of
renting cloud infrastructure. One self-contained binary. Your device, your bot, your business.

> **Status:** Public **testnet** release (`v0.2.0`). Hive runs as a **rApp** on the Reality
> testnet (global L0 genesis: `http://143.110.227.9:9000`). WhatsApp commerce, the local
> dashboard, and M-Pesa STK push are wired end-to-end; Reality snapshot submission has been
> verified against the testnet node.

---

## How it fits together

Hive is a **rApp** (Reality application). Each bot instance is its **own rApp**, identified by
its own key. At runtime hive captures business state (orders, vouchers, revenue) and its rApp
L0 submits signed **state-channel snapshots** up to the Reality global L0, where they're
included in global consensus.

```
Your device (laptop / phone / node)
├── WhatsApp connection (whatsapp-rust)
├── Bot engine (routing + conversation state)
├── Config + handlers (YAML-driven: menu, messages, payments)
├── Local web dashboard (admin panel, localhost only)
├── SQLite (orders, vouchers, payments, sessions)
└── rApp L0 submitter (src/network) ── signed snapshots ──▶ Reality global L0
```

Before the network accepts an instance's snapshots, that instance must be **registered
on-chain** with a `createDeployAppTransaction`. Deployment is done with **Reality's own
tooling** (keytool + wallet CLI) — hive is the app being posted, not the deployer. See
**[docs/DEPLOY.md](docs/DEPLOY.md)**.

---

## Install

**Prebuilt binary:**

```bash
curl -fsSL https://raw.githubusercontent.com/kalkiboru111/hive/main/setup.sh | bash
```

**From source** (Rust toolchain pinned in `rust-toolchain.toml`):

```bash
git clone https://github.com/kalkiboru111/hive
cd hive
cargo build --release
# binary at target/release/hive
```

---

## Quick start

### Builders (non-technical)

**👉 [FOR_BUILDERS.md](FOR_BUILDERS.md)** walks through everything. The fast path:

```bash
./hive wizard my-business     # answer a few questions, config is generated
./hive run my-business/       # scan the QR with WhatsApp — bot is live
```

### Developers

```bash
./hive init --template food-delivery my-bot   # scaffold from a template
nano my-bot/config.yaml                         # customize menu / messages
./hive run my-bot/                              # run the bot (+ dashboard)
```

---

## Usage

| Command | What it does |
|---------|--------------|
| `hive init <path> [--template <name>]` | Scaffold a new bot project (blank or from a template). |
| `hive wizard <path>` | Interactive setup — pick a business type and answer a few questions. |
| `hive templates` | List the available templates. |
| `hive run <path> [--phone <number>]` | Start the bot (and the dashboard if enabled). Shows a QR to pair WhatsApp; `--phone` uses pair-code auth instead. |
| `hive dashboard <path>` | Start only the admin dashboard (no WhatsApp), served at `http://localhost:<dashboard.port>` (default `8080`). |

> Deploying your rApp to the testnet is a separate step done with Reality's tooling —
> see [docs/DEPLOY.md](docs/DEPLOY.md). There is intentionally no `hive deploy` command.

---

## Templates

```bash
./hive templates
./hive init --template salon-booking my-salon
```

| Template | For |
|----------|-----|
| `food-delivery` | Restaurants, street food, home kitchens |
| `salon-booking` | Hair salons, barbers, spas |
| `event-tickets` | Concerts, workshops, classes |
| `tutoring` | Private lessons, test prep |
| `voucher-store` | Gift cards, loyalty programs |
| `community-store` | Co-ops, farmer's markets |
| `customer-support` | Help desk, ticketing |
| `real-estate` | Property listings, viewings |

Localized variants also ship: `food-delivery-mpesa` (Kenya / M-Pesa) and
`food-delivery-swahili`. Each template has pre-filled menu items, messages, and settings.

---

## Features

- **Config-driven** — define your bot in YAML, no coding required.
- **Menu & ordering** — product catalogs, cart building, order lifecycle, admin alerts.
- **Vouchers** — generate and redeem voucher codes.
- **M-Pesa payments 🇰🇪** — STK Push + payment webhooks ([guide](docs/MPESA_INTEGRATION.md)).
  _Note: B2C refunds are implemented in code but **not enabled in this release**._
- **Local web dashboard** — orders, payments, stats, analytics, reconciliation, ledger export
  (served on localhost).
- **Reality rApp** — signed state-channel snapshots submitted to the Reality network.
- **Single binary** — no Docker, no npm, no JVM to run the bot. Download and go.

---

## Deploy to the testnet

Each hive bot is its own rApp. To register yours and have its snapshots accepted:

1. Generate/own an rApp key with Reality's **keytool**.
2. Publish (or reference) the hive binary and **post a `createDeployAppTransaction`** with the
   Reality **wallet CLI** to `http://143.110.227.9:9000/transactions`.
3. Point hive at the **same key** (`network.identity_key_hex`) so its snapshot signatures match
   the deployed rApp address, and set `network.enabled: true`.

Full step-by-step (with the helper script `scripts/deploy.sh`):
**[docs/DEPLOY.md](docs/DEPLOY.md)**.

---

## Reality Network integration — how it works

1. **Identity** — each instance uses an rApp key (generated locally, or set via
   `network.identity_key_hex` to the key you deployed with).
2. **State capture** — orders/vouchers/revenue are summarized (PII-free; orders are hashed).
3. **Serialization** — the snapshot is encoded with MessagePack.
4. **Signing** — signed with the instance's secp256k1 key.
5. **Submission** — `POST {l0_url}/state-channels/{address}/snapshot`, with bounded retry and a
   chain-head persisted across restarts.
6. **Consensus** — once the rApp is deployed, the global L0 validates and includes the snapshot;
   the global ordinal advances.

Technical detail: **[docs/REALITY_INTEGRATION.md](docs/REALITY_INTEGRATION.md)**.

---

## Development

```bash
cargo build --release     # build
cargo test                # run tests
cargo clippy --all-targets
```

Run the Reality integration example against a node:

```bash
REALITY_URL=http://143.110.227.9:9000 cargo run --example test_reality
```

---

## Documentation

- **[FOR_BUILDERS.md](FOR_BUILDERS.md)** — non-technical introduction.
- **[docs/QUICKSTART.md](docs/QUICKSTART.md)** — minimal setup.
- **[docs/BUILDERS_GUIDE.md](docs/BUILDERS_GUIDE.md)** — full walkthrough, tips, FAQ.
- **[docs/DEPLOY.md](docs/DEPLOY.md)** — deploy your rApp to the testnet.
- **[docs/REALITY_INTEGRATION.md](docs/REALITY_INTEGRATION.md)** — snapshot architecture.
- **[docs/MPESA_INTEGRATION.md](docs/MPESA_INTEGRATION.md)** — M-Pesa setup.
- Internal design notes and historical test reports live in [docs/internal/](docs/internal/).

---

## Roadmap

**Shipped (v0.2.0):** WhatsApp pairing (QR + pair-code), menu & ordering, vouchers, local
dashboard, M-Pesa STK push + webhooks, Reality rApp snapshot submission, 8+ templates.

**In progress / planned:**
- Multi-language conversations — translation infrastructure exists (`src/i18n`) for 7 languages
  but is **not yet wired into the bot's message flow**.
- M-Pesa B2C refunds (code exists; needs config wiring + enabling).
- Enhanced analytics/exports; SMS fallback; delivery tracking.
- WhatsApp Business API tier for multi-agent / 24-7 teams.

See **[docs/internal/SCALING_ANALYSIS.md](docs/internal/SCALING_ANALYSIS.md)** for where the
current single-operator model fits and where it doesn't.

---

## Contributing

Issues and pull requests are welcome — please open an issue describing the change first.

## License

MIT — see [LICENSE](LICENSE).

## Credits

- [whatsapp-rust](https://github.com/jlucaso1/whatsapp-rust) — WhatsApp Web protocol.
- [Reality Network](https://github.com/reality-foundation) — decentralized compute platform.

## Support

- Issues: [github.com/kalkiboru111/hive/issues](https://github.com/kalkiboru111/hive/issues)
