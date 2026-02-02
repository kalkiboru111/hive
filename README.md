# 🐝 Hive — WhatsApp Bot Framework for Reality Network

Build and run WhatsApp bots on decentralized infrastructure. No cloud. No monthly fees. Your device, your bot, your business.

## What is Hive?

Hive is a framework that lets anyone create a WhatsApp-based business — ordering systems, customer service, booking, vouchers — and host it on their own device via [Reality Network](https://realitynet.xyz). Zero cloud costs. Works on a laptop or phone.

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

## Example: Cloudy Deliveries

A food delivery bot for townships. See `examples/cloudy-deliveries/` for the full template.

```yaml
business:
  name: "Cloudy Deliveries"
  currency: "ZAR"
  welcome: "Welcome to Cloudy Deliveries! 🍔☁️"

menu:
  - name: "Kota"
    price: 35.00
    emoji: "🌯"
  - name: "Bunny Chow"
    price: 45.00
    emoji: "🍛"
```

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

## Config Reference

See [docs/config.md](docs/config.md) for the full configuration reference.

## Building from Source

```bash
git clone https://github.com/reality-foundation/hive
cd hive
cargo build --release
```

The release binary is optimized for size (~5-10MB) and runs on Linux, macOS, and Windows.

## License

MIT
