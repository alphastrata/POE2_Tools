use super::{GameMaterials, NodeScaling};
use crate::components::{NodeActive, NodeMarker};
use crate::nodes::PassiveTreeWrapper;

use bevy::prelude::*;
use bevy::text::FontStyle;

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

/// This system checks if any node has been hovered (inactive or active)
/// for >= 0.5s, then sets the text to show that node’s info.
/// It also moves the text entity to the mouse cursor in world space.
pub fn show_node_info(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,

    // We read from the hovered components you already have:
    hovered_inactive: Query<(&HoveredInactive, &NodeMarker)>,
    hovered_active: Query<(&HoveredActive, &NodeMarker)>,

    // The text entity
    mut hover_text_query: Query<(&mut Text, &mut Transform), With<NodeHoverText>>,

    // The passive tree data so we can look up node names, etc.
    tree: Res<PassiveTreeWrapper>,
) {
    // 0) Try to get the single text entity
    let Ok((mut text, mut text_tf)) = hover_text_query.get_single_mut() else {
        log::warn!("Found no text to mutate...");
        return;
    };

    // Reset to empty by default, in case no hovered node qualifies
    text.0.clear();
    log::trace!("HoverText reset");

    // 1) Query debugging: how many hovered_inactive and hovered_active are present?
    let count_inactive = hovered_inactive.iter().count();
    let count_active = hovered_active.iter().count();
    log::trace!(
        "hovered_inactive count={}, hovered_active count={}",
        count_inactive,
        count_active
    );

    // 2) Attempt to find an *inactive* hovered node that’s been hovered >= 0.5s
    let mut found_info = None;
    for (hovered_comp, marker) in hovered_inactive.iter() {
        log::trace!(
            "Checking INACTIVE node marker={:?}, timer.elapsed={:.3}",
            marker.0,
            hovered_comp.timer.elapsed_secs()
        );

        if hovered_comp.timer.elapsed_secs() >= 0.250 {
            if let Some(node) = tree.tree.nodes.get(&marker.0) {
                let info_str = format!("Node {}:\n{}", node.node_id, node.name);
                log::debug!("Found INACTIVE hovered node: {}", &info_str);
                found_info = Some(info_str);
                break;
            } else {
                log::debug!("No matching node in tree for marker={:?}", marker.0);
            }
        }
    }
    log::trace!("Finished scanning INACTIVE hovers.");

    // 3) If not found among inactive, check active
    if found_info.is_none() {
        log::trace!("inactive empty of hovers...");
        for (hovered_comp, marker) in hovered_active.iter() {
            log::trace!(
                "Checking ACTIVE node marker={:?}, timer.elapsed={:.3}",
                marker.0,
                hovered_comp.timer.elapsed_secs()
            );

            if hovered_comp.timer.elapsed_secs() >= 0.250 {
                if let Some(node) = tree.tree.nodes.get(&marker.0) {
                    let info_str = format!("Node {}:\n{}", node.node_id, node.name);
                    log::debug!("Found ACTIVE hovered node: {}", &info_str);
                    found_info = Some(info_str);
                    break;
                } else {
                    log::debug!("No matching node in tree for marker={:?}", marker.0);
                }
            }
        }
        log::trace!("Finished scanning ACTIVE hovers.");
    } else {
        // We found something in the inactive loop
        log::trace!("We found something inactive, skipping active check.");
    }

    // 4) If we found any hovered node info, set text
    if let Some(info_str) = &found_info {
        log::debug!("Setting text to: {}", info_str);
        text.0 = info_str.clone();
    } else {
        log::trace!("No hovered node found => text stays empty.");
    }

    // 5) Move the text to the mouse cursor in "world space"
    let window = windows.single();
    let (camera, cam_tf) = camera_query.single();
    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(world_pos) = camera.viewport_to_world_2d(cam_tf, cursor_pos) {
            log::trace!("Mouse pos : {}, {}", world_pos.x, world_pos.y);
            // Add some offset so it doesn’t block the cursor
            text_tf.translation = Vec3::new(world_pos.x + 20.0, world_pos.y + 20.0, 999.0);
        } else {
            log::trace!("viewport_to_world_2d failed??");
        }
    } else {
        // If there's no cursor (e.g. out of window), optionally clear text:
        log::warn!("We might be out of window?");
        text.0.clear();
    }
}
