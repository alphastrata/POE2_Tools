use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::{
    components::SearchMarker,
    events::{SaveCharacterReq, ShowSearch},
    resources::{CameraSettings, SearchState, ToggleUi, UserConfig},
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
    mut save_tx: EventWriter<SaveCharacterReq>,
    config: Res<UserConfig>,
    keys: Res<ButtonInput<KeyCode>>,
    searchstate: Res<SearchState>,
    settings: ResMut<CameraSettings>,
    mut toggle_ui: ResMut<ToggleUi>,
) {
    // Always allow open/close of searchbox and the arrows:
    if check_action_just_pressed(&config, "search_for_node_by_name", &keys) {
        searchbox_toggle.send(ShowSearch);
        log::trace!("Searchbox toggle sent");
    }

    if check_action(&config, "toggle_ui", &keys) {
        toggle_ui.flip();
    }

    if !settings.egui_has_lock {
        // Camera:
        if let Ok(mut transform) = camera_query.get_single_mut() {
            let mut movement = Vec3::ZERO;
            //TODO: scale const by projection
            if check_action(&config, "move_left", &keys) {
                movement.x -= settings.drag_sensitivity * 20.;
            }
            if check_action(&config, "move_right", &keys) {
                movement.x += settings.drag_sensitivity * 20.;
            }
            if check_action(&config, "move_up", &keys) {
                movement.y += settings.drag_sensitivity * 20.;
            }
            if check_action(&config, "move_down", &keys) {
                movement.y -= settings.drag_sensitivity * 20.;
            }
            transform.translation += movement;
        }
    }

    if check_action_just_pressed(&config, "camera_reset_home", &keys) {
        if let Ok(mut transform) = camera_query.get_single_mut() {
            transform.translation = Vec3::ZERO;
        }
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

    if check_action_just_pressed(&config, "save_active_charcter", &keys) {
        save_tx.send(SaveCharacterReq);
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
