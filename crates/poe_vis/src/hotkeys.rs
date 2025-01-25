use bevy::prelude::*;

use crate::resources::UserConfig;

pub struct HotkeysPlugin;

impl Plugin for HotkeysPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_input);
    }
}

fn handle_input(
    config: Res<UserConfig>,
    keys: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    // Handle other actions
    if check_action_just_pressed(&config, "camera_reset_home", &keys) {
        if let Ok(mut transform) = camera_query.get_single_mut() {
            transform.translation = Vec3::ZERO;
        }
    }

    if check_action_just_pressed(&config, "exit", &keys) {
        std::process::exit(0)
    }
}

// Helper function to check just-pressed keys
fn check_action_just_pressed(
    config: &UserConfig,
    action: &str,
    keys: &ButtonInput<KeyCode>,
) -> bool {
    config
        .controls
        .get(action)
        .map(|keys_str| {
            keys_str
                .iter()
                .filter_map(|k| UserConfig::key_from_string(k))
                .any(|kc| keys.just_pressed(kc))
        })
        .unwrap_or(false)
}
// Helper function to check held keys
fn check_action(config: &UserConfig, action: &str, keys: &ButtonInput<KeyCode>) -> bool {
    config
        .controls
        .get(action)
        .map(|keys_str| {
            keys_str
                .iter()
                .filter_map(|k| UserConfig::key_from_string(k))
                .any(|kc| keys.pressed(kc))
        })
        .unwrap_or(false)
}
