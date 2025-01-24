use bevy::prelude::*;
use poe_tree::calculate_world_position;
use poe_tree::PassiveTree;

use crate::components::EdgeActive;
use crate::components::EdgeInactive;
use crate::components::EdgeMarker;
use crate::components::NodeActive;
use crate::components::NodeInactive;
use crate::components::NodeMarker;
use crate::config::parse_hex_color;
use crate::config::UserConfig;

// Add Resource derive for PassiveTree
#[derive(Resource, Debug, Clone)]
pub struct PassiveTreeWrapper {
    pub tree: PassiveTree,
}

#[derive(Resource)]
pub struct NodeScaling {
    pub min_scale: f32,
    pub max_scale: f32,
    pub base_radius: f32,
}

/// Adjust each node’s `Transform.scale` so it doesn’t get too big or too small on screen.
pub fn adjust_node_sizes(
    camera_query: Query<&OrthographicProjection, With<Camera2d>>,
    mut node_query: Query<&mut Transform, With<NodeMarker>>,
) {
    if let Ok(projection) = camera_query.get_single() {
        // By default, a larger `projection.scale` means you are "zoomed out"
        // so items appear smaller on screen, and vice versa.

        // For example, if you want the node scale to be simply the inverse:
        //   "1.0 / projection.scale"
        // you can then clamp that to keep it from vanishingly small or huge:
        let unscaled = 1.0 / projection.scale;
        let final_scale = unscaled.clamp(0.02, 2.0);
        // tweak these clamp values to taste
        log::debug!("{}", final_scale);

        for mut transform in &mut node_query {
            transform.scale = Vec3::splat(final_scale);
        }
    }
}

pub fn spawn_nodes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<GameMaterials>,
    tree: Res<PassiveTreeWrapper>,
    scaling: Res<NodeScaling>,
) {
    let node_radius = scaling.base_radius;

    for (_, node) in tree.tree.nodes.iter() {
        let group = tree.tree.groups.get(&node.parent).unwrap();
        let (x, y) = calculate_world_position(group, node.radius, node.position);

        commands.spawn((
            Mesh2d(meshes.add(Circle::new(node_radius))),
            MeshMaterial2d(materials.node_base.clone()),
            Transform::from_translation(Vec3::new(x, y, 0.0)),
            NodeMarker(node.node_id),
            NodeInactive,
        ));
    }
}

pub fn spawn_edges(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<GameMaterials>,
    tree: Res<PassiveTreeWrapper>,
) {
    tree.tree.edges.iter().for_each(|edge| {
        let (start_node, end_node) = (
            tree.tree.nodes.get(&edge.start).unwrap(),
            tree.tree.nodes.get(&edge.end).unwrap(),
        );

        let (start_group, end_group) = (
            tree.tree.groups.get(&start_node.parent).unwrap(),
            tree.tree.groups.get(&end_node.parent).unwrap(),
        );

        let start_pos =
            calculate_world_position(start_group, start_node.radius, start_node.position);
        let end_pos = calculate_world_position(end_group, end_node.radius, end_node.position);
        let start = Vec2::new(start_pos.0, start_pos.1);
        let end = Vec2::new(end_pos.0, end_pos.1);

        let delta = end - start;
        let width = delta.length();
        let height = 20.0;
        let angle = delta.y.atan2(delta.x);
        let midpoint = start.lerp(end, 0.5);

        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(width, height))),
            MeshMaterial2d(materials.edge_base.clone()),
            EdgeMarker((edge.start, edge.end)),
            Transform::from_translation(midpoint.extend(-0.01))
                .with_rotation(Quat::from_rotation_z(angle)),
            EdgeInactive,
        ));
    });
}

pub fn highlight_starting_node(
    character: Res<crate::config::ActiveCharacter>,
    mut commands: Commands,
    node_query: Query<(Entity, &NodeMarker), With<NodeInactive>>,
) {
    // Find and activate the starting node
    for (entity, marker) in node_query.iter() {
        if marker.0 == character.character.starting_node {
            commands
                .entity(entity)
                .remove::<NodeInactive>()
                .insert(NodeActive);
        }
    }
}
// Update materials system
pub fn update_materials(
    materials: Res<GameMaterials>,
    mut materials_query: ParamSet<(
        Query<(&mut MeshMaterial2d<ColorMaterial>, Option<&NodeActive>), Changed<NodeActive>>,
        Query<(&mut MeshMaterial2d<ColorMaterial>, Option<&EdgeActive>), Changed<EdgeActive>>,
    )>,
) {
    // Handle nodes
    let mut node_query = materials_query.p0();
    for (mut material, is_active) in node_query.iter_mut() {
        material.0 = if is_active.is_some() {
            materials.node_activated.clone()
        } else {
            materials.node_base.clone()
        };
    }

    // Handle edges
    let mut edge_query = materials_query.p1();
    for (mut material, is_active) in edge_query.iter_mut() {
        material.0 = if is_active.is_some() {
            materials.edge_activated.clone()
        } else {
            materials.edge_base.clone()
        };
    }
}

#[derive(Resource)]
pub struct GameMaterials {
    // Node colors
    pub node_base: Handle<ColorMaterial>,
    pub node_attack: Handle<ColorMaterial>,
    pub node_mana: Handle<ColorMaterial>,
    pub node_dexterity: Handle<ColorMaterial>,
    pub node_intelligence: Handle<ColorMaterial>,
    pub node_strength: Handle<ColorMaterial>,
    pub node_activated: Handle<ColorMaterial>,

    // Edge colors
    pub edge_base: Handle<ColorMaterial>,
    pub edge_activated: Handle<ColorMaterial>,

    // UI colors
    pub background: Handle<ColorMaterial>,
    pub foreground: Handle<ColorMaterial>,
    pub red: Handle<ColorMaterial>,
    pub orange: Handle<ColorMaterial>,
    pub yellow: Handle<ColorMaterial>,
    pub green: Handle<ColorMaterial>,
    pub blue: Handle<ColorMaterial>,
    pub purple: Handle<ColorMaterial>,
    pub cyan: Handle<ColorMaterial>,
}
pub fn init_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<UserConfig>,
) {
    commands.insert_resource(GameMaterials {
        // Node materials
        node_base: materials.add(parse_hex_color(&config.colors["all_nodes"])),
        node_attack: materials.add(parse_hex_color(&config.colors["attack"])),
        node_mana: materials.add(parse_hex_color(&config.colors["mana"])),
        node_dexterity: materials.add(parse_hex_color(&config.colors["dexterity"])),
        node_intelligence: materials.add(parse_hex_color(&config.colors["intelligence"])),
        node_strength: materials.add(parse_hex_color(&config.colors["strength"])),
        node_activated: materials.add(parse_hex_color(&config.colors["activated_nodes"])),

        // Edge materials
        edge_base: materials.add(parse_hex_color(&config.colors["all_nodes"])),
        edge_activated: materials.add(parse_hex_color(&config.colors["activated_edges"])),

        // UI materials
        background: materials.add(parse_hex_color(&config.colors["background"])),
        foreground: materials.add(parse_hex_color(&config.colors["foreground"])),
        red: materials.add(parse_hex_color(&config.colors["red"])),
        orange: materials.add(parse_hex_color(&config.colors["orange"])),
        yellow: materials.add(parse_hex_color(&config.colors["yellow"])),
        green: materials.add(parse_hex_color(&config.colors["green"])),
        blue: materials.add(parse_hex_color(&config.colors["blue"])),
        purple: materials.add(parse_hex_color(&config.colors["purple"])),
        cyan: materials.add(parse_hex_color(&config.colors["cyan"])),
    });
}
// Add new Hovered component
#[derive(Component)]
pub struct Hovered {
    timer: Timer,
}
pub mod hover {
    use super::*;

    #[derive(Component)]
    pub struct NodeHoverText;

    #[derive(Component)]
    pub struct Hovered {
        pub timer: Timer,
        pub base_scale: f32,
    }

    // Handles initial hover detection and visual changes
    pub fn highlight_hovered_node(
        mut commands: Commands,
        mut node_query: Query<(
            Entity,
            &Transform,
            &mut MeshMaterial2d<ColorMaterial>,
            &NodeMarker,
            &GlobalTransform,
        )>,
        windows: Query<&Window>,
        camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
        game_materials: Res<GameMaterials>,
        scaling: Res<NodeScaling>,
    ) {
        let window = windows.single();
        let (camera, camera_transform) = camera_query.single();

        let Some(cursor_pos) = window.cursor_position() else { return };
        let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return };

        for (entity, transform, mut material, _marker, global_transform) in &mut node_query {
            let node_pos = global_transform.translation().truncate();
            let node_radius = scaling.base_radius * transform.scale.x;
            let is_hovered = world_pos.distance(node_pos) <= node_radius;

            if is_hovered {
                // Store initial base scale when hover starts
                commands.entity(entity).insert(Hovered {
                    timer: Timer::from_seconds(0.5, TimerMode::Once),
                    base_scale: transform.scale.x,
                });
                
                // Immediate color change
                material.0 = game_materials.orange.clone();
            }
        }
    }

    // Handles ongoing hover state and scaling
    pub fn handle_highlighted_node(
        mut commands: Commands,
        mut node_query: Query<(
            Entity,
            &mut Transform,
            &mut Hovered,
            &mut MeshMaterial2d<ColorMaterial>,
        )>,
        game_materials: Res<GameMaterials>,
        scaling: Res<NodeScaling>,
        time: Res<Time>,
    ) {
        for (entity, mut transform, mut hovered, mut material) in &mut node_query {
            // Update timer
            hovered.timer.tick(time.delta());

            // Calculate target scale relative to initial base scale
            let target_scale = (hovered.base_scale * 1.15)
                .clamp(scaling.min_scale, scaling.max_scale);
            
            // Apply scale if under limit
            if transform.scale.x < target_scale {
                transform.scale = Vec3::splat(target_scale);
            }

            // Cleanup if hover ends
            if hovered.timer.finished() {
                material.0 = game_materials.node_base.clone();
                commands.entity(entity).remove::<Hovered>();
            }
        }
    }

    // show_node_info remains similar but uses Hovered component
    pub fn show_node_info(
        hovered_nodes: Query<(&Hovered, &NodeMarker)>,
        mut hover_text_query: Query<&mut Text, With<NodeHoverText>>,
        tree: Res<PassiveTreeWrapper>,
    ) {
        let Ok(mut text) = hover_text_query.get_single_mut() else { return };

        let mut node_info = None;
        for (hovered, marker) in &hovered_nodes {
            if hovered.timer.elapsed_secs() >= 0.5 {
                if let Some(node) = tree.tree.nodes.get(&marker.0) {
                    node_info = Some(format!(
                        "Node {}\n{}",
                        marker.0,
                        &node.name
                    ));
                }
                break;
            }
        }

        text.0 = node_info.unwrap_or_default();
    }
}