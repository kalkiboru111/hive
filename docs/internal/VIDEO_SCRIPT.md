# 🎬 Video Script: "Hive in 5 Minutes — WhatsApp Bot, No Code"

**Target:** Non-technical builders (entrepreneurs, small business owners)  
**Length:** 5 minutes  
**Tone:** Friendly, fast-paced, action-focused  
**Style:** Screen recording + voiceover (no talking head needed)

---

## 🎬 INTRO (0:00 - 0:20)

**[Visual: Hive logo animation → WhatsApp icon + lightning bolt]**

**Voiceover:**

> "You want to run a business on WhatsApp. Take orders, book appointments, sell tickets — all through messages. No app. No website. No monthly fees.
>
> This is Hive. A WhatsApp bot framework that runs on YOUR device. In 5 minutes, you'll have a working bot. Let's go."

**[Visual: Screen recording begins — desktop, terminal open]**

---

## 📥 STEP 1: DOWNLOAD (0:20 - 0:45)

**[Visual: Browser showing GitHub releases page]**

**Voiceover:**

> "Step one: download Hive. It's a single file — about 5 megabytes. No installation, no Docker, no npm. Just one binary.
>
> I'm on Mac, so I'm grabbing `hive-macos-arm64`. Linux, Windows, even Android Termux — all supported.
>
> Download, done."

**[Visual: File downloading → appears in Downloads folder]**

**[Visual: Terminal opens, `cd ~/Downloads`, `chmod +x hive`, `./hive --version`]**

**Voiceover:**

> "Make it executable, test it. Version 0.1. We're good."

---

## 🏗️ STEP 2: CREATE BOT (0:45 - 1:30)

**[Visual: Terminal, `./hive templates`]**

**Voiceover:**

> "Hive comes with templates. Food delivery, salon booking, event tickets, tutoring — pick one that matches your business.
>
> I'm building a food delivery bot, so:"

**[Visual: Types `./hive init --template food-delivery my-restaurant`]**

**Voiceover:**

> "This creates a folder with a pre-filled config. All I need to do is customize it."

**[Visual: Folder appears, shows `config.yaml`]**

**Voiceover:**

> "Let's open the config."

**[Visual: Opens `config.yaml` in VS Code or nano]**

---

## ✏️ STEP 3: EDIT CONFIG (1:30 - 2:45)

**[Visual: Scrolling through config.yaml, highlighting sections]**

**Voiceover:**

> "This is the ONLY file you edit. No code. Just plain text.
>
> **Business info** — name, currency, welcome message. I'll change this to 'Mama's Kitchen,' currency to KES (Kenyan shillings).
>
> **Menu** — list your items. Name, price, emoji, description. I'll update these to Ugali, Chapati, Pilau — typical Kenyan dishes.
>
> Copy, paste, edit. Simple.
>
> **Admin numbers** — this is YOUR WhatsApp number. Put it here with the country code. This is where order notifications go.
>
> **Delivery settings** — fee, estimated time. I'll set 50 KES delivery, 30-45 minutes.
>
> **Messages** — customize what customers see. 'Order confirmed,' 'Order delivered,' all that. You can tweak these later.
>
> Save. Done."

**[Visual: Saves file, closes editor]**

---

## ▶️ STEP 4: RUN THE BOT (2:45 - 3:15)

**[Visual: Back to terminal]**

**Voiceover:**

> "Now we run it."

**[Visual: Types `./hive run my-restaurant/`, presses Enter]**

**Voiceover:**

> "Hive loads the config, starts WhatsApp, and shows a QR code."

**[Visual: QR code appears in terminal]**

**Voiceover:**

> "Grab your phone. Open WhatsApp. Menu → Linked Devices → Link a Device. Scan."

**[Visual: Phone appears on screen, scans QR code (or animated overlay showing the process)]**

**Voiceover:**

> "And we're connected. Bot is live. Dashboard is running on port 8080."

**[Visual: Terminal shows "✅ WhatsApp connected! Bot is live!"]**

---

## 🧪 STEP 5: TEST IT (3:15 - 4:15)

**[Visual: Split screen — bot phone on left, customer phone on right]**

**Voiceover:**

> "Let's test. I'll message the bot from another phone."

**[Visual: Customer phone sends message to bot]**

**Bot response:**

```
Welcome to Mama's Kitchen! 🍛

Reply with a number:
1. 📋 View Menu
2. 📦 My Orders
3. ℹ️ About Us
```

**Voiceover:**

> "Instant reply. Good. Let's see the menu."

**[Visual: Customer sends "1"]**

**Bot response:**

```
📋 Menu:

1. Ugali & Sukuma - 150 KES 🥬
2. Chapati - 50 KES 🫓
3. Pilau - 200 KES 🍚

Reply "order [number]" to place an order!
```

**Voiceover:**

> "Perfect. Now let's place an order."

**[Visual: Customer sends "order 1"]**

**Bot response:**

```
✅ Order #1 confirmed!
📍 Please send your delivery address
⏱ Estimated delivery: 30-45 minutes
```

**[Visual: Admin phone buzzes — notification arrives]**

**Admin notification:**

```
🔔 New Order #1

1x Ugali & Sukuma - 150 KES

Total: 200 KES (incl. 50 KES delivery)
📍 [waiting for address]

Reply DONE 1 when delivered
```

**Voiceover:**

> "And I just got the order as admin. Customer sends their address, I prepare the food, deliver it, then mark it done."

**[Visual: Admin sends "DONE 1"]**

**Bot to customer:**

```
🎉 Order #1 has been delivered! Enjoy your meal! 😊
```

**Voiceover:**

> "Done. That's the full flow. Order, notification, delivery, confirmation."

---

## 🌐 STEP 6: DASHBOARD (4:15 - 4:45)

**[Visual: Browser opens, goes to `http://localhost:8080`]**

**Voiceover:**

> "Want to manage this without the terminal? Open the dashboard."

**[Visual: Dashboard loads — shows order list, menu editor, voucher creator]**

**Voiceover:**

> "See all orders. Edit your menu. Create voucher codes. Track sales. All from your browser.
>
> This is live. I can add a menu item right now — 'Samosas, 30 KES' — save, and it's instantly available to customers."

**[Visual: Adds menu item, saves, shows in bot]**

---

## 🚀 OUTRO (4:45 - 5:00)

**[Visual: Montage of different template examples — salon, events, tutoring]**

**Voiceover:**

> "That's Hive. One binary. One config file. Five minutes.
>
> Food delivery, salon booking, event tickets, tutoring — templates for everything. Download it. Try it. It's free, open-source, and runs on YOUR infrastructure.
>
> Links below. Now go build."

**[Visual: Fade to:
- **hive.realitynet.xyz** (or whatever the landing page is)
- **github.com/kalkiboru111/hive**
- **WhatsApp: Hive Builders Group** (QR code)]**

**[End screen: Hive logo + "Your device. Your bot. Your business."]**

---

## 🎥 PRODUCTION NOTES

### Screen Recording Tips
- **Use Zoom or OBS** to record at 1080p
- **Clean desktop** — hide personal files, tabs
- **Large terminal font** (18-20pt) so viewers can read
- **Speed up config editing** (2x speed, add captions)
- **Slow down key moments** (QR scan, order notification)

### Audio
- Clear, friendly voiceover (female or male, neutral accent)
- Background music: upbeat, minimal (royalty-free from Epidemic Sound or similar)
- Fade music during talking, bring it up during visual-only sections

### Phone Mockup
- Use device frame overlay (iPhone/Android mockup)
- Or screen record from actual phone (via ADB or QuickTime)

### Editing
- Fast cuts — no dead air
- Add text overlays for key commands (e.g., `./hive run my-restaurant/`)
- Highlight sections of config.yaml with zoom or colored box
- Add checkmarks ✅ when steps complete

### Thumbnail
- Bold text: "WhatsApp Bot in 5 Minutes"
- Show split screen: WhatsApp logo + terminal
- Include "No Code" badge
- Bright, high-contrast colors

### Platforms
- YouTube (primary)
- Twitter/X (clip highlights)
- TikTok/Reels (60-second version)
- WhatsApp Status (for viral spread in target markets)

---

## 📊 METRICS TO TRACK

- **Views** (target: 10k in first month)
- **Click-through to GitHub** (releases page)
- **WhatsApp group joins** (builder community)
- **Issue reports** ("stuck at step X" = guide needs improvement)

---

## 🌍 LOCALIZED VERSIONS

After English, consider:
- **Swahili** (Kenya, Tanzania, Uganda)
- **Afrikaans** (South Africa)
- **Portuguese** (Brazil, Angola, Mozambique)
- **Hindi** (India)
- **Spanish** (Latin America)

Same script, translated voiceover, culturally relevant examples (e.g., "Jollof Rice" for Nigeria, "Bunny Chow" for South Africa).

---

**Total production time estimate:** 2-3 days (scripting, recording, editing, publishing)  
**ROI:** High — video is permanent marketing asset, drives organic adoption

🐝 **Let's make this the definitive "how to build a WhatsApp bot" video.**
