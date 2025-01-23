//!$ crates/poe_vis/src/config.rs
use bevy::{color::Color, input::keyboard::Key, prelude::KeyCode};
use poe_tree::character::Character;
use std::collections::HashMap;

// Update color parsing to use non-deprecated methods
pub fn parse_hex_color(col_str: &str) -> Color {
    if col_str.starts_with('#') && col_str.len() == 7 {
        let hex = u32::from_str_radix(&col_str[1..7], 16).unwrap_or(0x808080);
        Color::srgb_u8(
            ((hex >> 16) & 0xFF) as u8,
            ((hex >> 8) & 0xFF) as u8,
            (hex & 0xFF) as u8,
        )
    } else {
        Color::srgb(0.48, 0.48, 0.48) // Fallback color (gray)
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

    pub fn key_from_string(key_str: &str) -> Option<KeyCode> {
        match key_str.to_lowercase().as_str() {
            // Alphabet keys
            "a" => Some(KeyCode::KeyA),
            "b" => Some(KeyCode::KeyB),
            "c" => Some(KeyCode::KeyC),
            "d" => Some(KeyCode::KeyD),
            "e" => Some(KeyCode::KeyE),
            "f" => Some(KeyCode::KeyF),
            "g" => Some(KeyCode::KeyG),
            "h" => Some(KeyCode::KeyH),
            "i" => Some(KeyCode::KeyI),
            "j" => Some(KeyCode::KeyJ),
            "k" => Some(KeyCode::KeyK),
            "l" => Some(KeyCode::KeyL),
            "m" => Some(KeyCode::KeyM),
            "n" => Some(KeyCode::KeyN),
            "o" => Some(KeyCode::KeyO),
            "p" => Some(KeyCode::KeyP),
            "q" => Some(KeyCode::KeyQ),
            "r" => Some(KeyCode::KeyR),
            "s" => Some(KeyCode::KeyS),
            "t" => Some(KeyCode::KeyT),
            "u" => Some(KeyCode::KeyU),
            "v" => Some(KeyCode::KeyV),
            "w" => Some(KeyCode::KeyW),
            "x" => Some(KeyCode::KeyX),
            "y" => Some(KeyCode::KeyY),
            "z" => Some(KeyCode::KeyZ),

            // Arrow keys
            "arrowup" => Some(KeyCode::ArrowUp),
            "arrowdown" => Some(KeyCode::ArrowDown),
            "arrowleft" => Some(KeyCode::ArrowLeft),
            "arrowright" => Some(KeyCode::ArrowRight),

            // Special keys
            "space" => Some(KeyCode::Space),
            "enter" => Some(KeyCode::Enter),
            "escape" => Some(KeyCode::Escape),
            "backspace" => Some(KeyCode::Backspace),
            "tab" => Some(KeyCode::Tab),
            "home" => Some(KeyCode::Home),
            "end" => Some(KeyCode::End),
            "pageup" => Some(KeyCode::PageUp),
            "pagedown" => Some(KeyCode::PageDown),
            "insert" => Some(KeyCode::Insert),
            "delete" => Some(KeyCode::Delete),

            // Fallback for unknown keys
            _ => None,
        }
    }
}

mod tests {

    #[test]
    fn can_parse_config() {
        use crate::config::UserConfig;
        let _config: UserConfig = UserConfig::load_from_file("../../data/user_config.toml");
    }
}
