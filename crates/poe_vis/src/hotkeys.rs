use bevy::prelude::*;

use crate::{
    components::SearchMarker,
    events::ShowSearch,
    resources::{CameraSettings, SearchState, UserConfig},
};

pub struct HotkeysPlugin;

impl Plugin for HotkeysPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_input);

        log::debug!("Hotkeys plugin enabled");
    }
}

fn handle_input(
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    mut searchbox_toggle: EventWriter<ShowSearch>,
    config: Res<UserConfig>,
    keys: Res<ButtonInput<KeyCode>>,
    searchstate: Res<SearchState>,
    settings: Res<CameraSettings>,
) {
    if check_action_just_pressed(&config, "camera_reset_home", &keys) {
        if let Ok(mut transform) = camera_query.get_single_mut() {
            transform.translation = Vec3::ZERO;
        }
    }
    if check_action_just_pressed(&config, "search_for_node_by_name", &keys) {
        searchbox_toggle.send(ShowSearch);
        log::trace!("Searchbox toggle sent");
    }

    if check_action_just_pressed(&config, "exit", &keys) {
        match !searchstate.open {
            true => std::process::exit(0),
            false => {
                // close the searchbox
                searchbox_toggle.send(ShowSearch);
            }
        }
    }

    // Camera:
    if let Ok(mut transform) = camera_query.get_single_mut() {
        let mut movement = Vec3::ZERO;
        if check_action(&config, "move_left", &keys) {
            movement.x -= settings.drag_sensitivity;
        }
        if check_action(&config, "move_right", &keys) {
            movement.x += settings.drag_sensitivity;
        }
        if check_action(&config, "move_up", &keys) {
            movement.y += settings.drag_sensitivity;
        }
        if check_action(&config, "move_down", &keys) {
            movement.y -= settings.drag_sensitivity;
        }
        transform.translation += movement;
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
