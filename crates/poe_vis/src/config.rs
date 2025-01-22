//!$ crates/poe_vis/src/config.rs
use egui::Key;
use poe_tree::character::Character;
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

#[derive(Debug, serde::Deserialize, Default)]
pub struct UserConfig {
    pub colors: HashMap<String, String>,
    pub controls: HashMap<String, Vec<String>>,

    #[serde(skip_deserializing)]
    #[serde(default)]
    pub character: Character,
}

impl UserConfig {
    pub fn load_from_file(path: &str) -> Self {
        let config_str = std::fs::read_to_string(path).expect("Unable to read config file");
        log::trace!("{}", &config_str);
        toml::from_str(&config_str).unwrap()
    }

    pub fn key_from_string(key_str: &str) -> Option<Key> {
        match key_str.to_lowercase().as_str() {
            "a" => Some(Key::A),
            "b" => Some(Key::B),
            "c" => Some(Key::C),
            "d" => Some(Key::D),
            "e" => Some(Key::E),
            "f" => Some(Key::F),
            "g" => Some(Key::G),
            "h" => Some(Key::H),
            "i" => Some(Key::I),
            "j" => Some(Key::J),
            "k" => Some(Key::K),
            "l" => Some(Key::L),
            "m" => Some(Key::M),
            "n" => Some(Key::N),
            "o" => Some(Key::O),
            "p" => Some(Key::P),
            "q" => Some(Key::Q),
            "r" => Some(Key::R),
            "s" => Some(Key::S),
            "t" => Some(Key::T),
            "u" => Some(Key::U),
            "v" => Some(Key::V),
            "w" => Some(Key::W),
            "x" => Some(Key::X),
            "y" => Some(Key::Y),
            "z" => Some(Key::Z),
            "0" => Some(Key::Num0),
            "1" => Some(Key::Num1),
            "2" => Some(Key::Num2),
            "3" => Some(Key::Num3),
            "4" => Some(Key::Num4),
            "5" => Some(Key::Num5),
            "6" => Some(Key::Num6),
            "7" => Some(Key::Num7),
            "8" => Some(Key::Num8),
            "9" => Some(Key::Num9),
            "arrowup" => Some(Key::ArrowUp),
            "arrowdown" => Some(Key::ArrowDown),
            "arrowleft" => Some(Key::ArrowLeft),
            "arrowright" => Some(Key::ArrowRight),
            "space" => Some(Key::Space),
            "enter" => Some(Key::Enter),
            "escape" => Some(Key::Escape),
            "backspace" => Some(Key::Backspace),
            "tab" => Some(Key::Tab),
            "home" => Some(Key::Home),
            "end" => Some(Key::End),
            "pageup" => Some(Key::PageUp),
            "pagedown" => Some(Key::PageDown),
            "insert" => Some(Key::Insert),
            "delete" => Some(Key::Delete),
            _ => None,
        }
    }

    pub fn mapped_controls(&self) -> HashMap<String, Key> {
        self.controls
            .iter()
            .filter_map(|(action, keys)| {
                keys.first()
                    .and_then(|k| Self::key_from_string(k))
                    .map(|key| (action.clone(), key))
            })
            .collect()
    }
}

mod tests {

    #[test]
    fn can_parse_config() {
        use crate::config::UserConfig;
        let _config: UserConfig = UserConfig::load_from_file("../../data/user_config.toml");
    }
}
