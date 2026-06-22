# 🐝 For Builders — Start Here

**You don't need to be a developer to build with Hive.**

This is your starting point. Everything you need to go from "I want a WhatsApp bot" to "My bot is live" — in under 10 minutes.

---

## 🎯 What You're Building

A **WhatsApp bot** that runs on YOUR device (no cloud needed). It can:

- Take orders (food delivery, products)
- Book appointments (salon, tutoring, viewings)
- Sell tickets (events, classes, workshops)
- Handle support (tickets, FAQs, inquiries)
- Create & redeem vouchers
- Track everything in a dashboard

**All through WhatsApp messages. No app. No website. No monthly fees.**

---

## 🚀 Quickest Path (5 Minutes)

### Use the Wizard

```bash
# Download Hive (one binary, ~5MB)
# See: https://github.com/kalkiboru111/hive/releases

# Make it executable (Mac/Linux)
chmod +x hive

# Run the wizard
./hive wizard my-business
```

**Answer 4 questions:**
1. What type of business? (food, salon, events, etc.)
2. Business name?
3. Currency?
4. Your WhatsApp number?

**Done.** Your bot config is ready.

```bash
# Run it
./hive run my-business/
```

**Scan the QR code with WhatsApp. Your bot is live.**

---

## 📚 Full Documentation

- **[Builder's Guide](docs/BUILDERS_GUIDE.md)** — Complete walkthrough with screenshots, tips, FAQ
- **[Deploy to the testnet](docs/DEPLOY.md)** — register your bot as a rApp on Reality
- **[Templates](#-templates)** — Pre-built configs for 8 common business types

---

## 🎨 Templates

Pre-configured bots for specific businesses. Just customize and go.

### See All Templates

```bash
./hive templates
```

### Use a Template

```bash
./hive init --template food-delivery my-restaurant
./hive init --template salon-booking my-salon
./hive init --template event-tickets my-events
```

**Available templates:**

| Template | Best For | Features |
|----------|----------|----------|
| **food-delivery** | Restaurants, street food, home kitchens | Menu, orders, delivery tracking |
| **salon-booking** | Hair salons, barbers, spas, nails | Service booking, appointments |
| **event-tickets** | Concerts, workshops, classes | Ticket sales, event listings |
| **tutoring** | Private lessons, test prep, language learning | Session booking, scheduling |
| **voucher-store** | Gift cards, loyalty programs | Digital vouchers, redemption |
| **community-store** | Co-ops, farmer's markets, local goods | Product catalog, pick-up/delivery |
| **customer-support** | Help desks, SaaS support | Ticket system, auto-replies |
| **real-estate** | Property agents, rental listings | Viewing scheduler, listings |

Each template includes:
- Pre-filled config with realistic examples
- Customizable messages
- Menu/service items ready to edit
- Admin notifications
- Dashboard support

---

## 🛠️ What You Edit

**One file:** `config.yaml`

It's plain text. No code. Just:

```yaml
business:
  name: "Your Business Name"
  currency: "USD"
  welcome: "Welcome message here"

menu:
  - name: "Item 1"
    price: 10.00
    emoji: "🍕"
```

**That's it.** Change the values, add items, save. Restart the bot. Changes are live.

---

## 📖 Learn by Example

### Example 1: Food Delivery Bot

```bash
./hive init --template food-delivery mama-kitchen
cd mama-kitchen
```

**Edit `config.yaml`:**
- Change business name to "Mama's Kitchen"
- Update currency to KES
- Replace menu items with your dishes
- Add your WhatsApp number to `admin_numbers`

**Run:**
```bash
../hive run .
```

**Test:**
- Message bot → get welcome
- Reply "1" → see menu
- "order 1" → place order
- Admin gets notification

**See full guide:** [docs/BUILDERS_GUIDE.md](docs/BUILDERS_GUIDE.md#example-food-delivery)

---

### Example 2: Salon Booking

```bash
./hive init --template salon-booking my-salon
```

**Edit to add services:**
```yaml
menu:
  - name: "Women's Haircut"
    price: 45.00
    emoji: "💇‍♀️"
    description: "Wash, cut, blow-dry (60 min)"
```

**Run, scan, go live.**

---

## 💡 Tips for Success

### 1. Start with a Template
Don't build from scratch. Pick the closest template, customize it.

### 2. Test Before Announcing
Run through the full flow yourself:
- Place an order
- Check admin notification
- Mark it complete
- Verify customer gets confirmation

### 3. Keep It Simple
Start with 3-5 menu items, not 50. Test, iterate, expand.

### 4. Use the Dashboard
Open `http://localhost:8080` to manage orders, edit menu, create vouchers — all without touching the config.

### 5. Run on a Spare Device
Use an old phone or laptop so your main machine doesn't need to stay on 24/7.

---

## 🤝 Get Help

**Stuck? Have questions? Want to show off your bot?**

- **GitHub Issues:** [kalkiboru111/hive/issues](https://github.com/kalkiboru111/hive/issues)

---

## 🌍 Real-World Use Cases

### 🇰🇪 Kenya: Township Food Delivery
*"I run a food business from home. Hive lets me take orders via WhatsApp without paying for delivery apps."*

### 🇿🇦 South Africa: Community Market
*"Our co-op uses Hive to coordinate orders from local farmers. Everyone orders via WhatsApp, we deliver on Saturdays."*

### 🇳🇬 Nigeria: Tutoring Services
*"I'm a tutor. Hive handles my bookings, sends reminders, and tracks payments. All through WhatsApp."*

### 🇮🇳 India: Event Tickets
*"We sell tickets to local meetups. Hive sends QR codes, tracks attendance, handles promo codes."*

---

## 📈 Next Level

Once you're live and running:

### 🔗 Deploy to the Reality testnet

Each Hive bot is its own **rApp**. Register yours on the Reality testnet so its activity is
recorded on-chain. This is done with Reality's keytool + wallet tooling — there is no
`hive publish` command. Full steps: **[docs/DEPLOY.md](docs/DEPLOY.md)**.

---

### 💰 Accept Payments

**M-Pesa (Kenya) works today** — STK Push + payment webhooks. Customers pay in-chat and orders
auto-confirm. See **[docs/MPESA_INTEGRATION.md](docs/MPESA_INTEGRATION.md)**.
*(B2C refunds exist in code but aren't enabled in this release.)*

---

### 🌐 Multi-Language

Translation infrastructure for 7 languages is built in (`src/i18n`), but isn't yet wired into
the live conversation flow — coming in a future release.

---

## 🐝 Your Turn

Download Hive. Pick a template. Edit the config. Run it. Scan the QR. Your bot is live.

**It's that simple.**

---

**Questions? Feedback? Built something cool?**  
👉 [Open an issue](https://github.com/kalkiboru111/hive/issues)

🐝 **Hive — Your device. Your bot. Your business.**
