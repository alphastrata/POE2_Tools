// In your controls module (controls/mod.rs)
use bevy::prelude::*;

use crate::config::UserConfig;

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        let config: UserConfig = UserConfig::load_from_file("data/user_config.toml");

        app.insert_resource(config)
            .add_systems(Update, handle_input);
    }
}

fn handle_input(
    config: Res<UserConfig>,
    keys: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    // Handle camera movement
    let mut direction = Vec2::ZERO;

    // Check configured keys for each movement action
    if check_action(&config, "move_left", &keys) {
        direction.x -= 1.0;
    }
    if check_action(&config, "move_right", &keys) {
        direction.x += 1.0;
    }
    if check_action(&config, "move_up", &keys) {
        direction.y += 1.0;
    }
    if check_action(&config, "move_down", &keys) {
        direction.y -= 1.0;
    }

    // Apply camera movement
    if let Ok(mut transform) = camera_query.get_single_mut() {
        let speed = 10.0; // Adjust as needed
        transform.translation += (direction * speed).extend(0.0);
    }

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

// Helper function to check held keys
fn check_action(config: &UserConfig, action: &str, keys: &ButtonInput<KeyCode>) -> bool {
    config
        .controls
        .get(action)
        .and_then(|keys_str| {
            Some(
                keys_str
                    .iter()
                    .filter_map(|k| UserConfig::key_from_string(k))
                    .any(|kc| keys.pressed(kc)),
            )
        })
        .unwrap_or(false)
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
        .and_then(|keys_str| {
            Some(
                keys_str
                    .iter()
                    .filter_map(|k| UserConfig::key_from_string(k))
                    .any(|kc| keys.just_pressed(kc)),
            )
        })
        .unwrap_or(false)
}
