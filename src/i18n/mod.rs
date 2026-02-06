//! Multi-language support for Hive bots.
//!
//! Provides translations for common bot messages in multiple languages.
//! Auto-detects user language from first message or allows manual selection.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    English,
    Swahili,    // Kenya, Tanzania, Uganda
    Afrikaans,  // South Africa
    Portuguese, // Brazil, Angola, Mozambique
    Hindi,      // India
    Spanish,    // Latin America
    French,     // West Africa
}

impl Language {
    /// Parse language from ISO code (e.g., "en", "sw", "af")
    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_lowercase().as_str() {
            "en" => Some(Language::English),
            "sw" => Some(Language::Swahili),
            "af" => Some(Language::Afrikaans),
            "pt" => Some(Language::Portuguese),
            "hi" => Some(Language::Hindi),
            "es" => Some(Language::Spanish),
            "fr" => Some(Language::French),
            _ => None,
        }
    }

    /// Get ISO code for this language
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Swahili => "sw",
            Language::Afrikaans => "af",
            Language::Portuguese => "pt",
            Language::Hindi => "hi",
            Language::Spanish => "es",
            Language::French => "fr",
        }
    }

    /// Get display name in the language itself
    pub fn native_name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Swahili => "Kiswahili",
            Language::Afrikaans => "Afrikaans",
            Language::Portuguese => "Português",
            Language::Hindi => "हिन्दी",
            Language::Spanish => "Español",
            Language::French => "Français",
        }
    }
}

/// Translation key for common bot messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TranslationKey {
    Welcome,
    ViewMenu,
    MyOrders,
    RedeemVoucher,
    AboutUs,
    OrderConfirmed,
    OrderDelivered,
    InvalidChoice,
    MenuEmpty,
    OrderPlaced,
    ThankYou,
    ChooseLanguage,
}

/// Translation provider
pub struct Translations {
    data: HashMap<(Language, TranslationKey), String>,
}

impl Translations {
    /// Create new translations instance with default translations
    pub fn new() -> Self {
        let mut data = HashMap::new();

        // English
        data.insert((Language::English, TranslationKey::Welcome), "Welcome! 👋".to_string());
        data.insert((Language::English, TranslationKey::ViewMenu), "📋 View Menu".to_string());
        data.insert((Language::English, TranslationKey::MyOrders), "📦 My Orders".to_string());
        data.insert((Language::English, TranslationKey::RedeemVoucher), "🎟️ Redeem Voucher".to_string());
        data.insert((Language::English, TranslationKey::AboutUs), "ℹ️ About Us".to_string());
        data.insert((Language::English, TranslationKey::OrderConfirmed), "✅ Order confirmed!".to_string());
        data.insert((Language::English, TranslationKey::OrderDelivered), "🎉 Order delivered! Enjoy!".to_string());
        data.insert((Language::English, TranslationKey::InvalidChoice), "❌ Invalid choice. Please try again.".to_string());
        data.insert((Language::English, TranslationKey::MenuEmpty), "No items available right now.".to_string());
        data.insert((Language::English, TranslationKey::OrderPlaced), "Your order has been placed!".to_string());
        data.insert((Language::English, TranslationKey::ThankYou), "Thank you! 😊".to_string());
        data.insert((Language::English, TranslationKey::ChooseLanguage), "Choose your language / Chagua lugha / Kies jou taal".to_string());

        // Swahili
        data.insert((Language::Swahili, TranslationKey::Welcome), "Karibu! 👋".to_string());
        data.insert((Language::Swahili, TranslationKey::ViewMenu), "📋 Angalia Menyu".to_string());
        data.insert((Language::Swahili, TranslationKey::MyOrders), "📦 Maagizo Yangu".to_string());
        data.insert((Language::Swahili, TranslationKey::RedeemVoucher), "🎟️ Tumia Vocha".to_string());
        data.insert((Language::Swahili, TranslationKey::AboutUs), "ℹ️ Kuhusu Sisi".to_string());
        data.insert((Language::Swahili, TranslationKey::OrderConfirmed), "✅ Agizo limethibitishwa!".to_string());
        data.insert((Language::Swahili, TranslationKey::OrderDelivered), "🎉 Agizo limefikishwa! Furahia!".to_string());
        data.insert((Language::Swahili, TranslationKey::InvalidChoice), "❌ Chaguo batili. Tafadhali jaribu tena.".to_string());
        data.insert((Language::Swahili, TranslationKey::MenuEmpty), "Hakuna vitu vinavyopatikana sasa hivi.".to_string());
        data.insert((Language::Swahili, TranslationKey::OrderPlaced), "Agizo lako limewekwa!".to_string());
        data.insert((Language::Swahili, TranslationKey::ThankYou), "Asante! 😊".to_string());
        data.insert((Language::Swahili, TranslationKey::ChooseLanguage), "Chagua lugha yako".to_string());

        // Afrikaans
        data.insert((Language::Afrikaans, TranslationKey::Welcome), "Welkom! 👋".to_string());
        data.insert((Language::Afrikaans, TranslationKey::ViewMenu), "📋 Sien Spyskaart".to_string());
        data.insert((Language::Afrikaans, TranslationKey::MyOrders), "📦 My Bestellings".to_string());
        data.insert((Language::Afrikaans, TranslationKey::RedeemVoucher), "🎟️ Wissel Koepon".to_string());
        data.insert((Language::Afrikaans, TranslationKey::AboutUs), "ℹ️ Oor Ons".to_string());
        data.insert((Language::Afrikaans, TranslationKey::OrderConfirmed), "✅ Bestelling bevestig!".to_string());
        data.insert((Language::Afrikaans, TranslationKey::OrderDelivered), "🎉 Bestelling afgelewer! Geniet!".to_string());
        data.insert((Language::Afrikaans, TranslationKey::InvalidChoice), "❌ Ongeldige keuse. Probeer asseblief weer.".to_string());
        data.insert((Language::Afrikaans, TranslationKey::MenuEmpty), "Geen items beskikbaar nie.".to_string());
        data.insert((Language::Afrikaans, TranslationKey::OrderPlaced), "Jou bestelling is geplaas!".to_string());
        data.insert((Language::Afrikaans, TranslationKey::ThankYou), "Dankie! 😊".to_string());
        data.insert((Language::Afrikaans, TranslationKey::ChooseLanguage), "Kies jou taal".to_string());

        // Portuguese
        data.insert((Language::Portuguese, TranslationKey::Welcome), "Bem-vindo! 👋".to_string());
        data.insert((Language::Portuguese, TranslationKey::ViewMenu), "📋 Ver Cardápio".to_string());
        data.insert((Language::Portuguese, TranslationKey::MyOrders), "📦 Meus Pedidos".to_string());
        data.insert((Language::Portuguese, TranslationKey::RedeemVoucher), "🎟️ Resgatar Cupom".to_string());
        data.insert((Language::Portuguese, TranslationKey::AboutUs), "ℹ️ Sobre Nós".to_string());
        data.insert((Language::Portuguese, TranslationKey::OrderConfirmed), "✅ Pedido confirmado!".to_string());
        data.insert((Language::Portuguese, TranslationKey::OrderDelivered), "🎉 Pedido entregue! Aproveite!".to_string());
        data.insert((Language::Portuguese, TranslationKey::InvalidChoice), "❌ Escolha inválida. Por favor, tente novamente.".to_string());
        data.insert((Language::Portuguese, TranslationKey::MenuEmpty), "Nenhum item disponível no momento.".to_string());
        data.insert((Language::Portuguese, TranslationKey::OrderPlaced), "Seu pedido foi feito!".to_string());
        data.insert((Language::Portuguese, TranslationKey::ThankYou), "Obrigado! 😊".to_string());
        data.insert((Language::Portuguese, TranslationKey::ChooseLanguage), "Escolha seu idioma".to_string());

        // Hindi
        data.insert((Language::Hindi, TranslationKey::Welcome), "स्वागत है! 👋".to_string());
        data.insert((Language::Hindi, TranslationKey::ViewMenu), "📋 मेनू देखें".to_string());
        data.insert((Language::Hindi, TranslationKey::MyOrders), "📦 मेरे ऑर्डर".to_string());
        data.insert((Language::Hindi, TranslationKey::RedeemVoucher), "🎟️ वाउचर रिडीम करें".to_string());
        data.insert((Language::Hindi, TranslationKey::AboutUs), "ℹ️ हमारे बारे में".to_string());
        data.insert((Language::Hindi, TranslationKey::OrderConfirmed), "✅ ऑर्डर की पुष्टि हो गई!".to_string());
        data.insert((Language::Hindi, TranslationKey::OrderDelivered), "🎉 ऑर्डर डिलीवर हो गया! आनंद लें!".to_string());
        data.insert((Language::Hindi, TranslationKey::InvalidChoice), "❌ अमान्य विकल्प। कृपया पुनः प्रयास करें।".to_string());
        data.insert((Language::Hindi, TranslationKey::MenuEmpty), "अभी कोई आइटम उपलब्ध नहीं है।".to_string());
        data.insert((Language::Hindi, TranslationKey::OrderPlaced), "आपका ऑर्डर दिया गया है!".to_string());
        data.insert((Language::Hindi, TranslationKey::ThankYou), "धन्यवाद! 😊".to_string());
        data.insert((Language::Hindi, TranslationKey::ChooseLanguage), "अपनी भाषा चुनें".to_string());

        // Spanish
        data.insert((Language::Spanish, TranslationKey::Welcome), "¡Bienvenido! 👋".to_string());
        data.insert((Language::Spanish, TranslationKey::ViewMenu), "📋 Ver Menú".to_string());
        data.insert((Language::Spanish, TranslationKey::MyOrders), "📦 Mis Pedidos".to_string());
        data.insert((Language::Spanish, TranslationKey::RedeemVoucher), "🎟️ Canjear Cupón".to_string());
        data.insert((Language::Spanish, TranslationKey::AboutUs), "ℹ️ Acerca de Nosotros".to_string());
        data.insert((Language::Spanish, TranslationKey::OrderConfirmed), "✅ ¡Pedido confirmado!".to_string());
        data.insert((Language::Spanish, TranslationKey::OrderDelivered), "🎉 ¡Pedido entregado! ¡Disfruta!".to_string());
        data.insert((Language::Spanish, TranslationKey::InvalidChoice), "❌ Opción inválida. Por favor, inténtalo de nuevo.".to_string());
        data.insert((Language::Spanish, TranslationKey::MenuEmpty), "No hay artículos disponibles ahora.".to_string());
        data.insert((Language::Spanish, TranslationKey::OrderPlaced), "¡Tu pedido ha sido realizado!".to_string());
        data.insert((Language::Spanish, TranslationKey::ThankYou), "¡Gracias! 😊".to_string());
        data.insert((Language::Spanish, TranslationKey::ChooseLanguage), "Elige tu idioma".to_string());

        // French
        data.insert((Language::French, TranslationKey::Welcome), "Bienvenue! 👋".to_string());
        data.insert((Language::French, TranslationKey::ViewMenu), "📋 Voir le Menu".to_string());
        data.insert((Language::French, TranslationKey::MyOrders), "📦 Mes Commandes".to_string());
        data.insert((Language::French, TranslationKey::RedeemVoucher), "🎟️ Utiliser un Coupon".to_string());
        data.insert((Language::French, TranslationKey::AboutUs), "ℹ️ À Propos".to_string());
        data.insert((Language::French, TranslationKey::OrderConfirmed), "✅ Commande confirmée!".to_string());
        data.insert((Language::French, TranslationKey::OrderDelivered), "🎉 Commande livrée! Bon appétit!".to_string());
        data.insert((Language::French, TranslationKey::InvalidChoice), "❌ Choix invalide. Veuillez réessayer.".to_string());
        data.insert((Language::French, TranslationKey::MenuEmpty), "Aucun article disponible pour le moment.".to_string());
        data.insert((Language::French, TranslationKey::OrderPlaced), "Votre commande a été passée!".to_string());
        data.insert((Language::French, TranslationKey::ThankYou), "Merci! 😊".to_string());
        data.insert((Language::French, TranslationKey::ChooseLanguage), "Choisissez votre langue".to_string());

        Self { data }
    }

    /// Get translation for a key in a specific language
    pub fn get(&self, lang: Language, key: TranslationKey) -> Option<&str> {
        self.data.get(&(lang, key)).map(|s| s.as_str())
    }

    /// Get translation or fall back to English
    pub fn get_or_fallback(&self, lang: Language, key: TranslationKey) -> &str {
        self.get(lang, key)
            .or_else(|| self.get(Language::English, key))
            .unwrap_or("[missing translation]")
    }
}

impl Default for Translations {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translations_exist() {
        let t = Translations::new();
        
        // Test English
        assert_eq!(t.get_or_fallback(Language::English, TranslationKey::Welcome), "Welcome! 👋");
        
        // Test Swahili
        assert_eq!(t.get_or_fallback(Language::Swahili, TranslationKey::Welcome), "Karibu! 👋");
        
        // Test Portuguese
        assert_eq!(t.get_or_fallback(Language::Portuguese, TranslationKey::OrderConfirmed), "✅ Pedido confirmado!");
    }

    #[test]
    fn test_language_codes() {
        assert_eq!(Language::from_code("sw"), Some(Language::Swahili));
        assert_eq!(Language::from_code("en"), Some(Language::English));
        assert_eq!(Language::Swahili.code(), "sw");
    }
}
