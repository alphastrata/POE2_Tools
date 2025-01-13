use std::collections::HashMap;

pub fn parse_color(col_str: &str) -> egui::Color32 {
    // Parse color from hex string (e.g., "#FF0000")
    if col_str.starts_with('#') && col_str.len() == 7 {
        let r = u8::from_str_radix(&col_str[1..3], 16).unwrap_or(255);
        let g = u8::from_str_radix(&col_str[3..5], 16).unwrap_or(255);
        let b = u8::from_str_radix(&col_str[5..7], 16).unwrap_or(255);
        egui::Color32::from_rgb(r, g, b)
    } else {
        egui::Color32::GRAY // Fallback if parsing fails
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct UserConfig {
    pub colors: HashMap<String, String>,
    pub controls: Option<HashMap<String, Vec<String>>>,
}

impl UserConfig {
    pub fn load_from_file(path: &str) -> Self {
        let config_str = std::fs::read_to_string(path).expect("Unable to read config file");
        toml::from_str(&config_str).expect("Invalid TOML format")
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserCharacter {
    pub name: String,
    pub activated_node_ids: Vec<usize>,
    pub date_created: String,
}

impl UserCharacter {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            activated_node_ids: Vec::new(),
            date_created: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn save_to_toml(&self, path: &str) {
        let serialized = toml::to_string(self).expect("Failed to serialize character to TOML");
        std::fs::write(path, serialized).expect("Failed to save character to TOML");
    }

    pub fn load_from_toml(path: &str) -> Option<Self> {
        let data = std::fs::read_to_string(path).ok()?;
        toml::from_str(&data).ok()
    }
}
