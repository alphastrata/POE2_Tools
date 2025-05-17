use bevy::{color::Color, prelude::*};
use poe_tree::{character::Character, type_wrappings::NodeId};
use std::collections::HashMap;

use crate::{
    components::{NodeActive, NodeInactive},
    resources::{ActiveCharacter, RootNode, UserConfig},
};

pub struct UserConfigPlugin;

impl Plugin for UserConfigPlugin {
    fn build(&self, app: &mut App) {
        let uc = UserConfig::load_from_file("data/user_config.toml");

        app.insert_resource(uc);

        log::debug!("UserConfig plugin enabled")
    }
}

impl crate::resources::UserConfig {
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
            "up_arrow" => Some(KeyCode::ArrowUp),
            "down_arrow" => Some(KeyCode::ArrowDown),
            "left_arrow" => Some(KeyCode::ArrowLeft),
            "right_arrow" => Some(KeyCode::ArrowRight),

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

            // "/" => Some(KeyCode::Backslash),
            "/" => Some(KeyCode::Slash),

            // Fallback for unknown keys
            _ => None,
        }
    }
}

mod tests {
    use crate::resources::UserConfig;

    #[test]
    fn can_parse_config() {
        let _config: UserConfig = UserConfig::load_from_file("../../data/user_config.toml");
    }
}
