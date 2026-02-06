# 🐝 Hive — WhatsApp Bot Framework for Reality Network

Build and run WhatsApp bots on decentralized infrastructure. No cloud. No monthly fees. Your device, your bot, your business.

## What is Hive?

Hive is a framework that lets anyone create a WhatsApp-based business — ordering systems, customer service, booking, vouchers — and host it on their own device via [Reality Network](https://realitynet.xyz). Zero cloud costs. Works on a laptop or phone.

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

## Documentation

- **[For Builders (Non-Technical)](FOR_BUILDERS.md)** — Start here if you're new
- **[Builder's Guide](docs/BUILDERS_GUIDE.md)** — Full walkthrough with examples, tips, FAQ
- **[Video Tutorial](docs/VIDEO_SCRIPT.md)** — 5-minute screencast (production script)
- **[Quickstart](docs/QUICKSTART.md)** — Minimal setup guide
- **[Config Reference](docs/config.md)** — Full configuration options *(coming soon)*

## Building from Source

```bash
git clone https://github.com/reality-foundation/hive
cd hive
cargo build --release
```

The release binary is optimized for size (~5-10MB) and runs on Linux, macOS, and Windows.

## License

MIT
