use super::{GameMaterials, NodeScaling};
use crate::components::{NodeActive, NodeMarker};
use crate::nodes::PassiveTreeWrapper;
use bevy::prelude::*;
use bevy::winit::cursor;

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
        // The text component
        Text::new(""),
        // Font configuration
        TextFont {
            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size: 22.0,
            ..default()
        },
        // Layout configuration for UI
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0), // Default starting position, whenever this is actually populated with content it'll be overridden.
            top: Val::Px(0.0),  // Default starting position
            ..default()
        },
        NodeHoverText, // Your custom marker
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
    hovered: Query<(&Hovered, &NodeMarker, Option<&NodeActive>)>,
    mut hover_text_query: Query<(&mut Node, &mut Text), With<NodeHoverText>>,
    tree: Res<PassiveTreeWrapper>,
) {
    // Attempt to get the hover text's Node and Text components
    let Ok((mut node, mut text)) = hover_text_query.get_single_mut() else {
        log::warn!("Found no UI node to update...");
        return;
    };

    // Clear the text initially
    text.0.clear();

    // Check if there's hovered node information
    let mut found_info: Option<String> = None;
    for (_hovered, marker, _maybe_active) in hovered.iter() {
        if let Some(node_info) = tree.tree.nodes.get(&marker.0) {
            found_info = Some(format!("Node {}:\n{}", node_info.node_id, node_info.name));
            break;
        }
    }

    // Update the text content if we found information
    if let Some(info) = found_info {
        log::debug!("Setting text to: {}", info);
        text.0 = info;
    }

    // Update the node's position in screen space
    if let Some(cursor_pos) = windows.single().cursor_position() {
        log::debug!("Cursor Position: {:?}", cursor_pos);

        node.left = Val::Px(cursor_pos.x);
        node.top = Val::Px(cursor_pos.y);
    }
}
