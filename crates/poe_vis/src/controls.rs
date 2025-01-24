// In your controls module (controls/mod.rs)
use bevy::prelude::*;

use crate::{
    components::{NodeActive, NodeInactive, NodeMarker},
    config::UserConfig,
};

pub struct KeyboardControlsPlugin;

impl Plugin for KeyboardControlsPlugin {
    fn build(&self, app: &mut App) {
        let config: UserConfig = UserConfig::load_from_file("data/user_config.toml");

        app.insert_resource(config)
            .add_systems(Update, handle_input);
    }
}

pub fn handle_node_clicks(
    mut commands: Commands,
    mut drag_state: ResMut<crate::camera::DragState>,
    root: Res<crate::config::RootNode>,
    mut click_events: EventReader<Pointer<Down>>,
    nodes_query: Query<
        (
            Entity,
            &NodeMarker,
            Option<&NodeInactive>,
            Option<&NodeActive>,
        ),
        Or<(With<NodeInactive>, With<NodeActive>)>,
    >,
) {
    for event in click_events.read() {
        if let Ok((entity, marker, inactive, active)) = nodes_query.get(event.target) {
            drag_state.active = false; // so that we don't mess up ppl's camera when activating/deactivating nodes.
            match (inactive, active) {
                (Some(_), None) => {
                    // Activate if root exists
                    if root.0.is_some() {
                        commands
                            .entity(entity)
                            .remove::<NodeInactive>()
                            .insert(NodeActive);
                    }
                }
                (None, Some(_)) => {
                    // Deactivate regardless of root
                    commands
                        .entity(entity)
                        .remove::<NodeActive>()
                        .insert(NodeInactive);
                }
                _ => unreachable!(),
            }
        }
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
