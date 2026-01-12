//! Locale loading and text retrieval.

use serde::Deserialize;
use std::collections::HashMap;

/// Supported locales.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Locale {
    #[default]
    English,
    Japanese,
}

impl Locale {
    /// Returns the locale code string.
    pub fn code(&self) -> &'static str {
        match self {
            Locale::English => "en",
            Locale::Japanese => "ja",
        }
    }

    /// Creates a Locale from a language code string.
    pub fn from_code(code: &str) -> Option<Locale> {
        match code.to_lowercase().as_str() {
            "en" | "english" => Some(Locale::English),
            "ja" | "jp" | "japanese" => Some(Locale::Japanese),
            _ => None,
        }
    }
}

/// Raw structure for parsing locale TOML files.
#[derive(Debug, Deserialize)]
struct LocaleData {
    #[serde(default)]
    items: HashMap<String, String>,
    #[serde(default)]
    machines: HashMap<String, String>,
    #[serde(default)]
    ui: HashMap<String, String>,
    #[serde(default)]
    readings: HashMap<String, String>,
}

/// Provides localized text retrieval.
#[derive(Debug, Clone, PartialEq)]
pub struct Localizer {
    items: HashMap<String, String>,
    machines: HashMap<String, String>,
    ui: HashMap<String, String>,
    readings: HashMap<String, String>,
}

impl Localizer {
    /// Creates a new Localizer from TOML content.
    ///
    /// # Arguments
    /// * `toml_content` - The TOML file content as a string
    ///
    /// # Returns
    /// A Result containing the Localizer or an error message.
    pub fn new(toml_content: &str) -> Result<Self, String> {
        let data: LocaleData = toml::from_str(toml_content)
            .map_err(|e| format!("Failed to parse locale file: {}", e))?;

        Ok(Localizer {
            items: data.items,
            machines: data.machines,
            ui: data.ui,
            readings: data.readings,
        })
    }

    /// Creates an empty Localizer (fallback only).
    pub fn empty() -> Self {
        Localizer {
            items: HashMap::new(),
            machines: HashMap::new(),
            ui: HashMap::new(),
            readings: HashMap::new(),
        }
    }

    /// Gets the localized name for an item.
    /// Falls back to the item ID if no translation exists.
    pub fn get_item(&self, item_id: &str) -> String {
        self.items
            .get(item_id)
            .cloned()
            .unwrap_or_else(|| item_id.to_string())
    }

    /// Gets the reading (furigana) for sorting purposes.
    /// Falls back to the localized name if no reading exists.
    /// This is primarily used for Japanese locale to enable proper sorting.
    pub fn get_reading(&self, item_id: &str) -> String {
        self.readings
            .get(item_id)
            .cloned()
            .unwrap_or_else(|| item_id.to_string())
    }

    /// Gets the localized name for a machine.
    /// Falls back to the machine ID if no translation exists.
    pub fn get_machine(&self, machine_id: &str) -> String {
        self.machines
            .get(machine_id)
            .cloned()
            .unwrap_or_else(|| machine_id.to_string())
    }

    /// Gets a localized UI string.
    /// Falls back to the key if no translation exists.
    pub fn get_ui(&self, key: &str) -> String {
        self.ui.get(key).cloned().unwrap_or_else(|| key.to_string())
    }
}
