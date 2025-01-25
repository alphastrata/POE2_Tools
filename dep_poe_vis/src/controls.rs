// In your controls module (controls/mod.rs)
use bevy::prelude::*;

use crate::{
    components::{Hovered, NodeActive, NodeInactive, NodeMarker},
    config::UserConfig,
    consts::DEFAULT_HOVER_FADE_TIME,
    materials::GameMaterials,
};

pub struct KeyboardControlsPlugin;

impl Plugin for KeyboardControlsPlugin {
    fn build(&self, app: &mut App) {
        let config: UserConfig = UserConfig::load_from_file("data/user_config.toml");

        app.insert_resource(config)
            .add_systems(Update, handle_input);
    }
}
