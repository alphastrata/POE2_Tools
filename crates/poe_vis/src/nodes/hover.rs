use super::{GameMaterials, NodeScaling};
use crate::components::{NodeActive, NodeMarker};
use crate::nodes::PassiveTreeWrapper;
use bevy::prelude::*;

// -------------------------------------------------------------------
// Components
// -------------------------------------------------------------------
#[derive(Component)]
pub struct NodeHoverText;

#[derive(Component)]
pub struct Hovered {
    pub timer: Timer,
    pub base_scale: f32,
}

/// If you want a default fade time, you can store it here or in NodeScaling
const DEFAULT_HOVER_FADE_TIME: f32 = 0.760;

// -------------------------------------------------------------------
// 0) Spawn Hover Text (once). Uses your 0.15.1 style API
// -------------------------------------------------------------------
pub fn spawn_hover_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        // A text with no content initially:
        Text::new(""),
        TextFont {
            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size: 42.0,
            ..default()
        },
        NodeHoverText,
    ));
}
pub fn hover_started(
    mut commands: Commands,
    mut over_events: EventReader<Pointer<Over>>,
    query_nodes: Query<(
        Entity,
        &NodeMarker,
        &Transform,
        Option<&Hovered>,
        Option<&NodeActive>, // so you can decide color
    )>,
    game_materials: Res<GameMaterials>,
) {
    for ev in over_events.read() {
        if let Ok((entity, marker, transform, maybe_hovered, maybe_active)) =
            query_nodes.get(ev.target)
        {
            if maybe_hovered.is_none() {
                commands.entity(entity).insert(Hovered {
                    timer: Timer::from_seconds(DEFAULT_HOVER_FADE_TIME, TimerMode::Once),
                    base_scale: transform.scale.x,
                });
                // Pick color based on active or not
                if maybe_active.is_some() {
                    commands
                        .entity(entity)
                        .insert(MeshMaterial2d(game_materials.cyan.clone()));
                } else {
                    commands
                        .entity(entity)
                        .insert(MeshMaterial2d(game_materials.orange.clone()));
                }
            }
        }
    }
}
pub fn hover_ended(
    mut commands: Commands,
    mut out_events: EventReader<Pointer<Out>>,
    query_nodes: Query<(Entity, Option<&NodeActive>, Option<&Hovered>)>,
    game_materials: Res<GameMaterials>,
) {
    for ev in out_events.read() {
        if let Ok((entity, maybe_active, maybe_hovered)) = query_nodes.get(ev.target) {
            if maybe_hovered.is_some() {
                commands.entity(entity).remove::<Hovered>();
                // revert color
                if maybe_active.is_some() {
                    commands
                        .entity(entity)
                        .insert(MeshMaterial2d(game_materials.node_activated.clone()));
                } else {
                    commands
                        .entity(entity)
                        .insert(MeshMaterial2d(game_materials.node_base.clone()));
                }
            }
        }
    }
}

pub fn handle_highlighted_active_nodes(
    mut node_query: Query<(&mut Transform, &Hovered), With<NodeActive>>,
    scaling: Res<NodeScaling>,
) {
    for (mut transform, hovered) in &mut node_query {
        let target_scale = hovered.base_scale * scaling.hover_multiplier;
        transform.scale = Vec3::splat(target_scale);
    }
}

pub fn handle_hovered_scaling(
    mut query: Query<(&mut Transform, &Hovered)>,
    scaling: Res<NodeScaling>,
) {
    for (mut transform, hovered) in &mut query {
        let target_scale = hovered.base_scale * scaling.hover_multiplier;
        transform.scale = Vec3::splat(target_scale);
    }
}

pub fn revert_hovered_nodes(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut Transform,
        &mut MeshMaterial2d<ColorMaterial>,
        &mut Hovered,
    )>,
    game_materials: Res<GameMaterials>,
) {
    for (entity, mut transform, mut material, mut hovered) in &mut query {
        hovered.timer.tick(time.delta());
        if hovered.timer.finished() {
            transform.scale = Vec3::splat(hovered.base_scale);
            // Possibly revert color using NodeActive, or just pick a default
            // For example:
            material.0 = game_materials.node_base.clone();
            commands.entity(entity).remove::<Hovered>();
        }
    }
}

pub fn show_node_info(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,

    hovered: Query<(&Hovered, &NodeMarker, Option<&NodeActive>)>,

    mut hover_text_query: Query<(&mut Text, &mut Transform), With<NodeHoverText>>,
    tree: Res<PassiveTreeWrapper>,
) {
    let Ok((mut text, mut text_tf)) = hover_text_query.get_single_mut() else {
        log::warn!("Found no text to mutate...");
        return;
    };
    text.0.clear();

    // We track if we found a hovered node in either query
    let mut found_info: Option<String> = None;

    // 2) If we didn't find anything yet, check Active
    if found_info.is_none() {
        for (hovered_comp, marker, _maybe_active) in hovered.iter() {
            //FIXME: there is something very skux about our timers we're inserting...
            // if hovered_comp.timer.elapsed_secs() >= 0.250 {
            if let Some(node) = tree.tree.nodes.get(&marker.0) {
                let info = format!("Node {}:\n{}", node.node_id, node.name);
                found_info = Some(info);
                break;
            }
            // }
        }
    }

    // 3) If we found any hovered node info, set text (and maybe panic for debug)
    if let Some(info_str) = &found_info {
        log::debug!("Setting text to: {}", info_str);
        text.0 = info_str.clone();
    }

    // 4) Move text near the cursor
    let window = windows.single();
    let (camera, cam_tf) = camera_query.single();
    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(world_pos) = camera.viewport_to_world_2d(cam_tf, cursor_pos) {
            text_tf.translation = Vec3::new(0.0, 0.0, 100.0);
        }
    }
}
