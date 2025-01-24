use super::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct NodeHoverText;

#[derive(Component)]
pub struct HoveredInactive {
    timer: Timer,
    base_scale: f32,
}

#[derive(Component)]
pub struct HoveredActive {
    timer: Timer,
    base_scale: f32,
}

// Hover system for INACTIVE nodes
pub fn highlight_hovered_inactive_nodes(
    mut commands: Commands,
    mut node_query: Query<
        (
            Entity,
            &Transform,
            &mut MeshMaterial2d<ColorMaterial>,
            &GlobalTransform,
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

    for (entity, transform, mut material, global_transform) in &mut node_query {
        let node_pos = global_transform.translation().truncate();
        let node_radius = scaling.base_radius * transform.scale.x;
        let is_hovered = world_pos.distance(node_pos) <= node_radius;

        if is_hovered {
            commands.entity(entity).insert(HoveredInactive {
                timer: Timer::from_seconds(0.100, TimerMode::Once),
                base_scale: transform.scale.x,
            });
            material.0 = game_materials.orange.clone();
        }
    }
}

// Hover system for ACTIVE nodes
pub fn highlight_hovered_active_nodes(
    mut commands: Commands,
    mut node_query: Query<
        (
            Entity,
            &Transform,
            &mut MeshMaterial2d<ColorMaterial>,
            &GlobalTransform,
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

    for (entity, transform, mut material, global_transform) in &mut node_query {
        let node_pos = global_transform.translation().truncate();
        let node_radius = scaling.base_radius * transform.scale.x;
        let is_hovered = world_pos.distance(node_pos) <= node_radius;

        if is_hovered {
            commands.entity(entity).insert(HoveredActive {
                timer: Timer::from_seconds(0.100, TimerMode::Once),
                base_scale: transform.scale.x,
            });
            material.0 = game_materials.cyan.clone();
        }
    }
}

/// Scales *inactive* nodes while hovered using `hover_multiplier`.
pub fn handle_highlighted_inactive_nodes(
    mut node_query: Query<(&mut Transform, &HoveredInactive), Without<NodeActive>>,
    scaling: Res<NodeScaling>,
) {
    for (mut transform, hovered) in &mut node_query {
        // Just multiply the original (hover start) scale by `hover_multiplier`
        let target_scale = hovered.base_scale * scaling.hover_multiplier;
        transform.scale = Vec3::splat(target_scale);
    }
}

/// Scales *active* nodes while hovered using `hover_multiplier`.
pub fn handle_highlighted_active_nodes(
    mut node_query: Query<(&mut Transform, &HoveredActive), With<NodeActive>>,
    scaling: Res<NodeScaling>,
) {
    for (mut transform, hovered) in &mut node_query {
        // We can use the base_radius as the reference for active nodes
        let target_scale = scaling.base_radius * scaling.hover_multiplier;
        transform.scale = Vec3::splat(target_scale);
    }
}

// Cleanup system for inactive hovers
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

    for (entity, mut transform, mut material, hovered, global_transform) in &mut node_query {
        let node_pos = global_transform.translation().truncate();
        let node_radius = scaling.base_radius * transform.scale.x;
        let is_hovered = world_pos.distance(node_pos) <= node_radius;

        if let Some(hovered) = hovered {
            if !is_hovered {
                transform.scale = Vec3::splat(hovered.base_scale);
                material.0 = game_materials.node_base.clone();
                commands.entity(entity).remove::<HoveredInactive>();
            }
        }
    }
}

/// Return to original size if Hover Timer expires (INACTIVE).
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
            // Revert to base_scale we had at hover start
            transform.scale = Vec3::splat(hovered.base_scale);
            // or if you prefer always returning to `scaling.base_radius`, do:
            // transform.scale = Vec3::splat(scaling.base_radius);

            // Revert color
            material.0 = game_materials.node_base.clone();
            commands.entity(entity).remove::<HoveredInactive>();
        }
    }
}

/// Return to original size if Hover Timer expires (ACTIVE).
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
            // Revert to the scale at the moment we started hovering, or back to base:
            transform.scale = Vec3::splat(hovered.base_scale);
            // Or use `scaling.base_radius` if that is your desired "active" size:
            // transform.scale = Vec3::splat(scaling.base_radius);

            // Revert color
            material.0 = game_materials.node_activated.clone();
            commands.entity(entity).remove::<HoveredActive>();
        }
    }
}

// Cleanup system for active hovers
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

    for (entity, mut transform, mut material, hovered, global_transform) in &mut node_query {
        let node_pos = global_transform.translation().truncate();
        let node_radius = scaling.base_radius * transform.scale.x;
        let is_hovered = world_pos.distance(node_pos) <= node_radius;

        if let Some(hovered) = hovered {
            if !is_hovered {
                transform.scale = Vec3::splat(hovered.base_scale);
                material.0 = game_materials.node_activated.clone();
                commands.entity(entity).remove::<HoveredActive>();
            }
        }
    }
}

// Hover tracking component (replaces custom Hovered)
#[derive(Component)]
pub struct NodeHoverState {
    timer: Timer,
    base_scale: f32,
}

pub fn spawn_hover_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        // Create a Text with multiple child spans.
        Text::new(""),
        TextFont {
            // This font is loaded and will be used instead of the default font.
            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size: 42.0,
            ..default()
        },
        NodeHoverText,
    ));
}

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
                info = format!("Node {}\n{}", marker.0, node.name,);
            }
            break;
        }
    }

    text.0 = info;
    //TODO: tf needs manipulating to be on the cursor... or ontop of the node?
}
