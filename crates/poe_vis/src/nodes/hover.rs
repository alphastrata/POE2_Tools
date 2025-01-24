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
pub struct HoveredInactive {
    pub timer: Timer,
    pub base_scale: f32,
}

#[derive(Component)]
pub struct HoveredActive {
    pub timer: Timer,
    pub base_scale: f32,
}

const DEFAULT_HOVER_FADE_TIME: f32 = 0.200; // in seconds

pub fn spawn_hover_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        // Create a Text with multiple child spans.
        Text::new(""),
        TextFont {
            // Load your custom font
            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size: 42.0,
            ..default()
        },
        NodeHoverText,
    ));
}

// -------------------------------------------------------------------
// 1) Hover detection for INACTIVE nodes
//    - We do an Option<&HoveredInactive> in the query
//    - If is_hovered && hovered_inactive.is_none(), insert component
// -------------------------------------------------------------------
pub fn highlight_hovered_inactive_nodes(
    mut commands: Commands,
    mut node_query: Query<
        (
            Entity,
            &Transform,
            &mut MeshMaterial2d<ColorMaterial>,
            &GlobalTransform,
            Option<&HoveredInactive>,
        ),
        (With<NodeMarker>, Without<NodeActive>),
    >,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    game_materials: Res<GameMaterials>,
    scaling: Res<NodeScaling>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_query.single();

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    for (entity, transform, mut material, global_transform, hovered_inactive_opt) in &mut node_query
    {
        let node_pos = global_transform.translation().truncate();
        let node_radius = scaling.base_radius * transform.scale.x;
        let is_hovered = world_pos.distance(node_pos) <= node_radius;

        // Only insert HoveredInactive if the node is hovered AND doesn't already have it
        if is_hovered && hovered_inactive_opt.is_none() {
            commands.entity(entity).insert(HoveredInactive {
                timer: Timer::from_seconds(DEFAULT_HOVER_FADE_TIME, TimerMode::Once),
                base_scale: transform.scale.x,
            });
            material.0 = game_materials.orange.clone();
        }
    }
}

// -------------------------------------------------------------------
// 2) Hover detection for ACTIVE nodes
//    - Similar approach, but we look for Option<&HoveredActive>
// -------------------------------------------------------------------
pub fn highlight_hovered_active_nodes(
    mut commands: Commands,
    mut node_query: Query<
        (
            Entity,
            &Transform,
            &mut MeshMaterial2d<ColorMaterial>,
            &GlobalTransform,
            Option<&HoveredActive>,
        ),
        (With<NodeActive>, Without<HoveredInactive>),
    >,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    game_materials: Res<GameMaterials>,
    scaling: Res<NodeScaling>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_query.single();

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    for (entity, transform, mut material, global_transform, hovered_active_opt) in &mut node_query {
        let node_pos = global_transform.translation().truncate();
        let node_radius = scaling.base_radius * transform.scale.x;
        let is_hovered = world_pos.distance(node_pos) <= node_radius;

        if is_hovered && hovered_active_opt.is_none() {
            commands.entity(entity).insert(HoveredActive {
                timer: Timer::from_seconds(DEFAULT_HOVER_FADE_TIME, TimerMode::Once),
                base_scale: transform.scale.x,
            });
            material.0 = game_materials.cyan.clone();
        }
    }
}

// -------------------------------------------------------------------
// 3) Scale systems for hovered nodes
//    - Called each frame, but we only scale once because base_scale stays fixed
// -------------------------------------------------------------------
pub fn handle_highlighted_inactive_nodes(
    mut node_query: Query<(&mut Transform, &HoveredInactive), Without<NodeActive>>,
    scaling: Res<NodeScaling>,
) {
    for (mut transform, hovered) in &mut node_query {
        // Each frame, set scale = base_scale * hover_multiplier
        // (No repeated accumulation, because base_scale was fixed on insertion)
        let target_scale = hovered.base_scale * scaling.hover_multiplier;
        transform.scale = Vec3::splat(target_scale);
    }
}

pub fn handle_highlighted_active_nodes(
    mut node_query: Query<(&mut Transform, &HoveredActive), With<NodeActive>>,
    scaling: Res<NodeScaling>,
) {
    for (mut transform, hovered) in &mut node_query {
        let target_scale = hovered.base_scale * scaling.hover_multiplier;
        transform.scale = Vec3::splat(target_scale);
    }
}

// -------------------------------------------------------------------
// 4) Cleanup systems for INACTIVE & ACTIVE
//    - If the cursor leaves, revert immediately
// -------------------------------------------------------------------
pub fn cleanup_inactive_hovers(
    mut commands: Commands,
    mut node_query: Query<
        (
            Entity,
            &mut Transform,
            &mut MeshMaterial2d<ColorMaterial>,
            Option<&HoveredInactive>,
            &GlobalTransform,
        ),
        Without<NodeActive>,
    >,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    game_materials: Res<GameMaterials>,
    scaling: Res<NodeScaling>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_query.single();

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    for (entity, mut transform, mut material, hovered_opt, global_transform) in &mut node_query {
        if let Some(hovered) = hovered_opt {
            let node_pos = global_transform.translation().truncate();
            let node_radius = scaling.base_radius * transform.scale.x;
            let is_hovered = world_pos.distance(node_pos) <= node_radius;

            if !is_hovered {
                // revert
                transform.scale = Vec3::splat(hovered.base_scale);
                material.0 = game_materials.node_base.clone();
                commands.entity(entity).remove::<HoveredInactive>();
            }
        }
    }
}

pub fn cleanup_active_hovers(
    mut commands: Commands,
    mut node_query: Query<
        (
            Entity,
            &mut Transform,
            &mut MeshMaterial2d<ColorMaterial>,
            Option<&HoveredActive>,
            &GlobalTransform,
        ),
        With<NodeActive>,
    >,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    game_materials: Res<GameMaterials>,
    scaling: Res<NodeScaling>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_query.single();

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    for (entity, mut transform, mut material, hovered_opt, global_transform) in &mut node_query {
        if let Some(hovered) = hovered_opt {
            let node_pos = global_transform.translation().truncate();
            let node_radius = scaling.base_radius * transform.scale.x;
            let is_hovered = world_pos.distance(node_pos) <= node_radius;

            if !is_hovered {
                // revert
                transform.scale = Vec3::splat(hovered.base_scale);
                material.0 = game_materials.node_activated.clone();
                commands.entity(entity).remove::<HoveredActive>();
            }
        }
    }
}

// -------------------------------------------------------------------
// 5) Revert if timer finishes (i.e. hovered for > .100s?)
// -------------------------------------------------------------------
pub fn revert_inactive_hovered_nodes(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<
        (
            Entity,
            &mut Transform,
            &mut MeshMaterial2d<ColorMaterial>,
            &mut HoveredInactive,
        ),
        Without<NodeActive>,
    >,
    game_materials: Res<GameMaterials>,
) {
    for (entity, mut transform, mut material, mut hovered) in &mut query {
        hovered.timer.tick(time.delta());
        if hovered.timer.finished() {
            // revert
            transform.scale = Vec3::splat(hovered.base_scale);
            material.0 = game_materials.node_base.clone();
            commands.entity(entity).remove::<HoveredInactive>();
        }
    }
}

pub fn revert_active_hovered_nodes(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<
        (
            Entity,
            &mut Transform,
            &mut MeshMaterial2d<ColorMaterial>,
            &mut HoveredActive,
        ),
        With<NodeActive>,
    >,
    game_materials: Res<GameMaterials>,
) {
    for (entity, mut transform, mut material, mut hovered) in &mut query {
        hovered.timer.tick(time.delta());
        if hovered.timer.finished() {
            // revert
            transform.scale = Vec3::splat(hovered.base_scale);
            material.0 = game_materials.node_activated.clone();
            commands.entity(entity).remove::<HoveredActive>();
        }
    }
}

// -------------------------------------------------------------------
// 6) Example "show_node_info" system, if you have NodeHoverState
//    or you want to unify the logic. (You can adapt your code.)
// -------------------------------------------------------------------

#[derive(Component)]
pub struct NodeHoverState {
    pub timer: Timer,
    pub base_scale: f32,
}

/// Example system that tries to display info after 0.5s
pub fn show_node_info(
    hovered_nodes: Query<(&NodeHoverState, &NodeMarker)>,
    mut hover_text_query: Query<(&mut Text, &mut Transform), With<NodeHoverText>>,
    tree: Res<PassiveTreeWrapper>,
) {
    let Ok((mut text, mut tf)) = hover_text_query.get_single_mut() else {
        return;
    };
    let mut info = String::new();

    for (hovered, marker) in &hovered_nodes {
        if hovered.timer.elapsed_secs() >= 0.5 {
            if let Some(node) = tree.tree.nodes.get(&marker.0) {
                info = format!("Node {}\n{}", marker.0, node.name);
            }
            break;
        }
    }

    text.0 = info;
    // TODO: position tf near the node or the cursor
}
