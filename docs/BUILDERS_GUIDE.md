# 🐝 Hive Builder's Guide

**Turn your WhatsApp into a business. No coding required.**

This guide walks you through building, launching, and managing your WhatsApp bot — step by step, with screenshots and examples.

---

## Who This Is For

- Small business owners
- Solo entrepreneurs
- Community organizers
- Anyone who wants to sell/book/coordinate via WhatsApp

**What you need:**
- A phone with WhatsApp
- A computer OR a second old phone (to run the bot)
- 10 minutes

**What you DON'T need:**
- Coding skills
- Cloud hosting
- Monthly subscription fees
- Payment processing accounts

---

## Step 1: Download Hive (2 minutes)

### Option A: Download Pre-Built Binary (Easiest)

1. Go to: **[releases page]** *(link TBD)*
2. Download for your platform:
   - **macOS (Apple Silicon):** `hive-macos-arm64`
   - **macOS (Intel):** `hive-macos-x86_64`
   - **Linux (64-bit):** `hive-linux-x86_64`
   - **Windows:** `hive-windows.exe`
   - **Android (Termux):** `hive-linux-arm64`

3. Open Terminal (Mac/Linux) or Command Prompt (Windows)

4. Make it executable (Mac/Linux only):
   ```bash
   chmod +x hive-macos-arm64
   mv hive-macos-arm64 hive
   ```

5. Test it:
   ```bash
   ./hive --version
   ```

   You should see: `Hive v0.1.0` (or similar)

**✅ Success?** Move to Step 2.

**❌ Problems?**
- "Command not found" → You're not in the right folder. Use `cd ~/Downloads`
- "Permission denied" → Run `chmod +x hive` again
- Still stuck? Ask in [Hive Builders WhatsApp Group] *(link TBD)*

---

### Option B: Build from Source (Advanced)

If you're comfortable with dev tools:

```bash
# Install Rust (if you don't have it)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/kalkiboru111/hive
cd hive
cargo build --release

# Binary is at: target/release/hive
./target/release/hive --version
```

---

## Step 2: Create Your Bot (3 minutes)

### Quick Start (Use a Template)

Hive includes pre-made templates for common businesses. Pick one:

```bash
./hive templates
```

**Available templates:**
- `food-delivery` — Restaurant, street food, home kitchen
- `salon-booking` — Hair salon, barber, spa, nails
- `event-tickets` — Concerts, workshops, classes
- `tutoring` — Private lessons, language learning, test prep
- `voucher-store` — Gift cards, community credits
- `community-store` — Co-op, farmer's market, local goods
- `customer-support` — Help desk, ticket system
- `real-estate` — Property listings, rental viewings

**Create from template:**

```bash
./hive init --template food-delivery my-restaurant
```

This creates a folder: `my-restaurant/` with a pre-filled `config.yaml`.

---

### Custom Start (Blank Template)

If you want to start from scratch:

```bash
./hive init my-business
```

---

## Step 3: Edit Your Config (5 minutes)

Open `my-restaurant/config.yaml` in any text editor (Notepad, TextEdit, VS Code, etc.)

### 🔹 Section 1: Business Info

```yaml
business:
  name: "Mama's Kitchen"        # Your business name
  currency: "KES"               # USD, EUR, KES, ZAR, etc.
  welcome: |                    # First message customers see
    Welcome to Mama's Kitchen! 🍛
    
    Reply with a number:
    1. 📋 View Menu
    2. 📦 My Orders
    3. ℹ️ About Us
  about: "Homemade Kenyan meals, delivered fresh to your door."
```

**💡 Tip:** The `|` after `welcome:` lets you write multi-line text.

---

### 🔹 Section 2: Menu / Products / Services

```yaml
menu:
  - name: "Ugali & Sukuma"      # Item name
    price: 150                  # Price (no currency symbol)
    emoji: "🥬"                 # Optional emoji (makes it pretty!)
    description: "Traditional ugali with sautéed greens"
  
  - name: "Chapati"
    price: 50
    emoji: "🫓"
    description: "Soft, fresh chapati (5 pieces)"
  
  - name: "Pilau"
    price: 200
    emoji: "🍚"
    description: "Spiced rice with chicken or beef"
```

**💡 Tips:**
- Copy/paste items to add more
- Keep descriptions short (one line)
- Price is in your currency (no decimals for KES, use `.00` for USD)
- Find emojis at: [emojipedia.org](https://emojipedia.org)

---

### 🔹 Section 3: Delivery (Optional)

```yaml
delivery:
  fee: 50                       # Delivery charge
  estimate_minutes: [30, 45]   # Estimated delivery time range
```

**Don't do delivery?** Delete this section or comment it out with `#`:

```yaml
# delivery:
#   fee: 50
```

---

### 🔹 Section 4: Admin Numbers (IMPORTANT!)

```yaml
admin_numbers:
  - "+254712345678"     # Your WhatsApp number (with country code!)
  - "+254798765432"     # Optional: second admin
```

**This is where order notifications go.** Use international format:
- Kenya: `+254...`
- USA: `+1...`
- South Africa: `+27...`
- Nigeria: `+234...`

---

### 🔹 Section 5: Custom Messages (Optional)

```yaml
messages:
  order_confirmed: "✅ Order #{id} confirmed! 📍 Send your location."
  order_delivered: "🎉 Order #{id} delivered! Enjoy! 😊"
```

**Placeholders you can use:**
- `{id}` → Order number
- `{items}` → List of items ordered
- `{total}` → Total price
- `{currency}` → Your currency (USD, KES, etc.)
- `{location}` → Customer's address/location
- `{estimate}` → Delivery time estimate

---

### 🔹 Section 6: Dashboard

```yaml
dashboard:
  port: 8080        # Web dashboard runs on this port
  enabled: true     # Set to false if you don't want the dashboard
```

**Dashboard lets you:**
- See all orders
- Edit menu items
- Create vouchers
- View analytics

Access it at: `http://localhost:8080` (from the same device running Hive)

---

### 🔹 Section 7: Reality Network (Optional)

```yaml
network:
  enabled: false   # turn on after deploying your rApp (see docs/DEPLOY.md)
  l0_url: "http://143.110.227.9:9000"   # Reality testnet L0
```

**What this does:**
- Records your bot's order/voucher state on the Reality testnet (decentralized, on-chain proof).

**Important:** each bot is its own **rApp** and must be deployed once before the testnet accepts
its snapshots. See **[DEPLOY.md](DEPLOY.md)**. Leave `enabled: false` until you've deployed.

---

## Step 4: Run Your Bot (1 minute)

```bash
./hive run my-restaurant/
```

**You should see:**

```
🐝 Hive v0.1.0
📂 Loaded config: my-restaurant/config.yaml
📱 Starting WhatsApp connection...
📷 QR Code:

█████████████████████████████████
█████████████████████████████████
███               ███           ██
███ ▄▄▄▄▄ █  ▄█ ███ ▄▄▄▄▄ ███
███ █   █ ██▄ █ ███ █   █ ███
███ █▄▄▄█ █ ▄▀█ ███ █▄▄▄█ ███
█████████████████████████████████
```

---

## Step 5: Pair with WhatsApp (1 minute)

1. Open WhatsApp on your phone
2. Tap **⋮** (menu) → **Linked Devices**
3. Tap **Link a Device**
4. Scan the QR code from your terminal

**✅ Success?** You'll see:

```
✅ WhatsApp connected!
📞 Logged in as: +254712345678
🌐 Dashboard running at: http://localhost:8080
🐝 Bot is live!
```

**Your bot is now running!** Leave the terminal open.

---

## Step 6: Test Your Bot (2 minutes)

1. **From another phone**, message your bot's WhatsApp number
2. You should get the welcome message
3. Reply `1` → See your menu
4. Reply `order 1` → Place an order
5. **Check your admin phone** → You get a notification!
6. Reply `DONE 1` → Customer gets "delivered" message

**Everything working?** 🎉 Your bot is live!

---

## Common Issues & Fixes

### ❌ QR code doesn't appear
- **Check:** Are you in the right folder? Run `ls` — you should see `config.yaml`
- **Fix:** `cd my-restaurant/` then `../hive run .`

### ❌ WhatsApp says "Couldn't link device"
- **Cause:** QR code expired (they last 30 seconds)
- **Fix:** Restart Hive, scan faster

### ❌ Bot doesn't respond to messages
- **Check:** Is the terminal still running? Did it crash?
- **Fix:** Look at the logs in `my-restaurant/logs/` — share in support group if stuck

### ❌ Admin notifications not arriving
- **Check:** Did you set `admin_numbers` correctly? Include country code?
- **Fix:** Edit `config.yaml`, restart Hive

### ❌ Dashboard shows "Connection refused"
- **Check:** Is Hive still running? Did it crash?
- **Fix:** Restart Hive. Check `dashboard.enabled: true` in config.

---

## Next Steps

### 📊 Use the Dashboard

Open in your browser: `http://localhost:8080`

**You can:**
- View all orders (pending, completed, cancelled)
- Edit menu items (add/remove/change prices)
- Create voucher codes
- See sales analytics
- Export data (CSV)

**Screenshot placeholder:**
```
[Dashboard showing order list, menu editor, voucher creator]
```

---

### 🎟️ Create Vouchers

**From Dashboard:**
1. Go to "Vouchers" tab
2. Click "Create New"
3. Set code (e.g., `WELCOME10`)
4. Set value (10% off or flat R50 discount)
5. Click Save

**From Bot (admin mode):**
Send to your bot:
```
voucher create WELCOME10 50
```

Creates a R50 discount code customers can redeem.

---

### 📱 Run on a Spare Phone

**Why?** So you don't need your computer running 24/7.

**How?**
1. Install [Termux](https://termux.dev) (Android)
2. Download `hive-linux-arm64`
3. Transfer your `config.yaml` to the phone
4. Run:
   ```bash
   ./hive run ~/my-restaurant/
   ```

5. Phone can stay plugged in, bot runs forever

---

### 🚀 Deploy to the Reality testnet

Each bot is its own **rApp**. Register yours on the testnet so its activity is recorded
on-chain. This uses Reality's keytool + wallet tooling (there is no `./hive publish` command).

**Full steps:** [DEPLOY.md](DEPLOY.md)

---

## Advanced: Multi-Language

Want to support multiple languages? Use conditional welcome messages:

```yaml
welcome: |
  Welcome! 🌍
  
  Choose language / Chagua lugha:
  1. English
  2. Swahili
```

Then use the bot's language detection to serve translated menus.

*(Full multi-language guide coming soon)*

---

## Get Help

- **WhatsApp Group:** [Hive Builders Support] *(link TBD)*
- **GitHub Issues:** [github.com/kalkiboru111/hive/issues](https://github.com/kalkiboru111/hive/issues)
- **Email:** support@realitynet.xyz

---

## Template Gallery

### 🍔 Food Delivery
*See: `templates/food-delivery.yaml`*

Best for: Restaurants, street food, home kitchens

**Key features:**
- Menu with prices, emojis, descriptions
- Delivery fee calculation
- Location requests
- Order tracking
- Admin notifications

**Try it:**
```bash
./hive init --template food-delivery my-kitchen
```

---

### 💇 Salon Booking
*See: `templates/salon-booking.yaml`*

Best for: Hair salons, barbers, spas, nail studios

**Key features:**
- Service menu with durations
- Booking confirmations
- Appointment reminders
- Gift vouchers

**Try it:**
```bash
./hive init --template salon-booking my-salon
```

---

### 🎟️ Event Tickets
*See: `templates/event-tickets.yaml`*

Best for: Concerts, workshops, classes, meetups

**Key features:**
- Event listings
- Ticket purchases
- QR code generation (coming soon)
- Promo codes
- Check-in system

**Try it:**
```bash
./hive init --template event-tickets my-events
```

---

### 📚 Tutoring
*See: `templates/tutoring.yaml`*

Best for: Private tutors, language teachers, test prep

**Key features:**
- Lesson booking
- Session scheduling
- Payment tracking
- Student progress notes

**Try it:**
```bash
./hive init --template tutoring my-tutoring
```

---

### 🎁 Voucher Store
*See: `templates/voucher-store.yaml`*

Best for: Gift cards, community credits, loyalty programs

**Key features:**
- Digital voucher sales
- Balance checking
- Redemption tracking
- Bonus tiers (buy $50, get $55)

**Try it:**
```bash
./hive init --template voucher-store my-vouchers
```

---

### 🌾 Community Store
*See: `templates/community-store.yaml`*

Best for: Co-ops, farmer's markets, local goods

**Key features:**
- Product catalog
- Pick-up or delivery
- Inventory tracking
- Vendor management (coming soon)

**Try it:**
```bash
./hive init --template community-store my-market
```

---

### 🆘 Customer Support
*See: `templates/customer-support.yaml`*

Best for: Small businesses, SaaS, service companies

**Key features:**
- Ticket submission
- Issue categorization
- Auto-replies
- SLA tracking

**Try it:**
```bash
./hive init --template customer-support my-support
```

---

### 🏡 Real Estate
*See: `templates/real-estate.yaml`*

Best for: Property agents, rental managers, vacation homes

**Key features:**
- Listing catalog with photos
- Viewing scheduler
- Application forms
- Referral tracking

**Try it:**
```bash
./hive init --template real-estate my-listings
```

---

## Tips for Success

### 1. **Start Small**
Don't list 50 menu items on day one. Start with 3-5, test, refine.

### 2. **Test Everything**
Before going live, test:
- Ordering flow
- Admin notifications
- Delivery messages
- Voucher codes

### 3. **Announce It**
Once live, tell your customers:
- Post on social media
- Share your WhatsApp number
- Offer a launch discount (first 10 orders 20% off)

### 4. **Monitor Logs**
If something breaks, check `my-business/logs/` — errors will show there.

### 5. **Keep It Running**
Use a spare device or keep your computer on. Or deploy to Reality Network for 24/7 uptime.

---

## FAQ

**Q: Does the customer need to install anything?**  
A: No. They just message your WhatsApp number. That's it.

**Q: How much does it cost to run?**  
A: Zero. Hive is free, open-source. Reality Network is decentralized (no hosting fees).

**Q: Can I use my main WhatsApp number?**  
A: No. You need a separate number for the bot (WhatsApp Business number works great).

**Q: What if I want to change the menu?**  
A: Edit `config.yaml`, restart Hive. Changes are live instantly. Or use the dashboard.

**Q: Can customers pay via WhatsApp?**  
A: Not yet. Payment integration (M-Pesa, UPI, crypto) coming Q2 2026.

**Q: Can I run multiple bots?**  
A: Yes. Create separate folders, each with its own config. Run them in separate terminals.

**Q: Is my data private?**  
A: Yes. All data is on your device (SQLite database). Reality Network sync is optional and encrypted.

**Q: What if my bot crashes?**  
A: Check `logs/`. Common issues: wrong config syntax, network disconnection. Auto-restart coming soon.

**Q: Can I customize the bot responses?**  
A: Yes. Edit the `messages:` section in `config.yaml`. Full customization guide coming soon.

---

## Next: Watch the Video

👉 **[Hive in 5 Minutes — Video Walkthrough]** *(link TBD)*

Watch a real setup, start to finish.

---

**Built something cool? Share it in the [Hive Builders group]!**

🐝 **Hive — Your device. Your bot. Your business.**
