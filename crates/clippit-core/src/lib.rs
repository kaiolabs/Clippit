pub mod config;
pub mod history;
pub mod storage;
pub mod types;
pub mod validator;

pub use config::Config;
pub use history::HistoryManager;
pub use types::{ClipboardEntry, ContentType};
pub use validator::ContentValidator;

// Initialize i18n
rust_i18n::i18n!("locales", fallback = "en");

// Re-export the t! macro for use in other crates
pub use rust_i18n::t;

/// Set the application language
pub fn set_language(lang: &str) {
    rust_i18n::set_locale(lang);
}
