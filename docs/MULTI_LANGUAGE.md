# 🌍 Multi-Language Support

Hive supports serving customers in multiple languages automatically.

---

## Supported Languages

| Language | Code | Native Name | Regions |
|----------|------|-------------|---------|
| English | `en` | English | Global |
| Swahili | `sw` | Kiswahili | Kenya, Tanzania, Uganda |
| Afrikaans | `af` | Afrikaans | South Africa |
| Portuguese | `pt` | Português | Brazil, Angola, Mozambique |
| Hindi | `hi` | हिन्दी | India |
| Spanish | `es` | Español | Latin America |
| French | `fr` | Français | West Africa |

---

## How It Works

### Automatic Detection
Hive detects the user's language from their first message (future feature).

### Manual Selection
Show a language picker in your welcome message:

```yaml
business:
  welcome: |
    Welcome! / Karibu! / Welkom! 👋
    
    Choose your language:
    1. English
    2. Kiswahili
    3. Afrikaans
    4. Português
    5. हिन्दी
    6. Español
    7. Français
```

---

## Using Translations in Code

### Rust API

```rust
use hive::i18n::{Language, Translations, TranslationKey};

let translations = Translations::new();

// Get translation
let welcome = translations.get_or_fallback(
    Language::Swahili,
    TranslationKey::Welcome
);

println!("{}", welcome); // "Karibu! 👋"
```

### Available Translation Keys

- `Welcome`
- `ViewMenu`
- `MyOrders`
- `RedeemVoucher`
- `AboutUs`
- `OrderConfirmed`
- `OrderDelivered`
- `InvalidChoice`
- `MenuEmpty`
- `OrderPlaced`
- `ThankYou`
- `ChooseLanguage`

---

## Example: Multi-Language Food Delivery Bot

```yaml
business:
  name: "Mama's Kitchen"
  currency: "KES"
  welcome: |
    Welcome! / Karibu! / Welkom! 👋
    
    Choose language / Chagua lugha / Kies taal:
    🇬🇧 1. English
    🇰🇪 2. Kiswahili
    🇿🇦 3. Afrikaans
  
  # Enable multi-language mode
  languages:
    enabled: true
    default: "en"
    supported: ["en", "sw", "af"]

menu:
  - name:
      en: "Ugali & Greens"
      sw: "Ugali na Sukuma"
      af: "Ugali en Groente"
    price: 150
    emoji: "🥬"
```

**Note:** Full multi-language config support (like the example above) is coming soon. Currently, translations are available in code but not yet wired into the config YAML.

---

## Translating Menu Items

### Simple Approach (Current)
Keep menu items in one language, let users choose language for system messages only:

```yaml
menu:
  - name: "Ugali & Sukuma"  # Swahili menu item
    price: 150
    emoji: "🥬"
```

### Future: Multi-Language Menu
```yaml
menu:
  - name:
      en: "Cornmeal & Greens"
      sw: "Ugali na Sukuma"
      af: "Mielliepap en Groente"
    description:
      en: "Traditional meal with sautéed greens"
      sw: "Chakula cha jadi na mboga zilizokangwa"
      af: "Tradisionele maaltyd met gesauteerde groente"
```

---

## Adding a New Language

### 1. Add to `Language` Enum

Edit `src/i18n/mod.rs`:

```rust
pub enum Language {
    English,
    Swahili,
    YourLanguage,  // Add here
}
```

### 2. Add Translations

Add translations in the `Translations::new()` method:

```rust
// Your Language
data.insert((Language::YourLanguage, TranslationKey::Welcome), 
    "Welcome in your language! 👋".to_string());
// ... add all translation keys
```

### 3. Add ISO Code

```rust
"xx" => Some(Language::YourLanguage),
```

### 4. Add Native Name

```rust
Language::YourLanguage => "Native Name",
```

### 5. Test

```bash
cargo test i18n
```

---

## Best Practices

### 1. **Keep It Simple**
Don't translate everything. Key system messages + language picker is enough.

### 2. **Use Emojis**
Emojis are universal. Use them in menus so language doesn't matter as much.

### 3. **Local Currency**
Use local currency codes (KES, ZAR, INR) so customers understand prices.

### 4. **Test with Native Speakers**
Have someone who speaks the language test your translations.

### 5. **Fallback to English**
If a translation is missing, English is used automatically.

---

## Roadmap

**Coming soon:**

- [ ] Auto-detect language from WhatsApp locale
- [ ] Multi-language config support (YAML structure above)
- [ ] Language switcher command ("type `lang sw` to switch to Swahili")
- [ ] Per-user language preferences (stored in database)
- [ ] Translation helpers for custom messages
- [ ] Community translation contributions (Crowdin integration?)

---

## Contributing Translations

Want to add a language? Open a PR!

**What we need:**
- Language name + ISO code
- Translations for all `TranslationKey` variants
- Native speaker review

**High-priority languages:**
- Zulu (South Africa)
- Xhosa (South Africa)
- Yoruba (Nigeria)
- Igbo (Nigeria)
- Hausa (Nigeria)
- Amharic (Ethiopia)
- Arabic (North Africa, Middle East)
- Bengali (Bangladesh, India)
- Indonesian (Indonesia)

---

## Example: Swahili Food Delivery Bot

**Config:**
```yaml
business:
  name: "Chakula cha Mama"
  currency: "KES"
  welcome: |
    Karibu Chakula cha Mama! 🍛
    
    Jibu na namba:
    1. 📋 Angalia Menyu
    2. 📦 Maagizo Yangu
    3. 🎟️ Tumia Vocha
    4. ℹ️ Kuhusu Sisi

menu:
  - name: "Ugali na Sukuma"
    price: 150
    emoji: "🥬"
    description: "Ugali na mboga za majani"
  
  - name: "Pilau"
    price: 200
    emoji: "🍚"
    description: "Wali wa pilau na nyama"

messages:
  order_confirmed: "✅ Agizo limethibitishwa! 📍 Tuma anwani yako."
  order_delivered: "🎉 Agizo limefikishwa! Furahia! 😊"
```

**Result:** A fully Swahili bot for Kenyan customers.

---

🐝 **Multi-language support makes Hive accessible to billions of non-English speakers worldwide.**
