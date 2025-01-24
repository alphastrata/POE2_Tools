use bevy::prelude::*;
use bevy::utils::HashSet;
use poe_tree::calculate_world_position;
use poe_tree::type_wrappings::NodeId;
use poe_tree::PassiveTree;

use crate::components::EdgeActive;
use crate::components::EdgeMarker;
use crate::components::NodeActive;
use crate::components::NodeMarker;
use crate::config::parse_hex_color;
use crate::config::UserConfig;

// Add Resource derive for PassiveTree
#[derive(Resource, Debug, Clone)]
pub struct PassiveTreeWrapper {
    pub tree: PassiveTree,
}

#[derive(Resource)]
struct NodeScaling {
    min_scale: f32,
    max_scale: f32,
    base_radius: f32,
}
// Plugin to display nodes
pub struct PoeVis;

/// Adjust each node’s `Transform.scale` so it doesn’t get too big or too small on screen.
fn adjust_node_sizes(
    camera_query: Query<&OrthographicProjection, With<Camera2d>>,
    mut node_query: Query<&mut Transform, With<NodeMarker>>,
) {
    // If you only have one main camera, just get_single()
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
impl Plugin for PoeVis {
    fn build(&self, app: &mut App) {
        app.insert_resource(NodeScaling {
            min_scale: 1.0,    // Nodes can shrink to 50% size
            max_scale: 4.0,    // Nodes can grow to 200% size
            base_radius: 40.0, // Should match your node radius
        })
        .add_plugins(crate::camera::PoeVisCameraPlugin)
        .add_systems(PreStartup, init_materials)
        .add_systems(Startup, (spawn_nodes, spawn_edges, adjust_node_sizes))
        .add_systems(
            Update,
            (handle_node_clicks, update_edge_activation, update_materials),
        );
    }
}

// Add the ActivatedMaterials resource definition
#[derive(Resource)]
struct ActivatedMaterials {
    node: Handle<ColorMaterial>,
    edge: Handle<ColorMaterial>,
}

fn spawn_nodes(
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
            NodeActive(false),
        ));
    }
}

fn spawn_edges(
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
            EdgeActive(false),
        ));
    });
}

fn handle_node_clicks(
    mut click_events: EventReader<Pointer<Down>>,
    mut node_query: Query<&mut NodeActive, With<NodeMarker>>,
) {
    for event in click_events.read() {
        // Access the target field directly from the event
        if let Ok(mut active) = node_query.get_mut(event.target) {
            active.0 = !active.0;
        }
    }
}

//---------UPDATES-----------//
// System 1: Update edge activation states
fn update_edge_activation(
    node_query: Query<(&NodeMarker, &NodeActive), Changed<NodeActive>>,
    mut edge_query: Query<(&EdgeMarker, &mut EdgeActive)>,
) {
    // Clear existing edge states
    for (_, mut active) in &mut edge_query {
        active.0 = false;
    }

    // Get active nodes
    let active_nodes: HashSet<NodeId> = node_query
        .iter()
        .filter(|(_, active)| active.0)
        .map(|(marker, _)| marker.0)
        .collect();

    // Update edges based on active nodes using EdgeMarker data
    for (edge_marker, mut active) in &mut edge_query {
        let (start, end) = edge_marker.0;
        active.0 = active_nodes.contains(&start) && active_nodes.contains(&end);
    }
}

fn update_materials(
    materials: Res<GameMaterials>,
    mut nodes: Query<(&NodeActive, &mut MeshMaterial2d<ColorMaterial>), Changed<NodeActive>>,
    mut edges: Query<(&EdgeActive, &mut MeshMaterial2d<ColorMaterial>), Changed<EdgeActive>>,
) {
    // Update nodes
    for (active, mut material) in &mut nodes {
        material.0 = if active.0 {
            materials.node_activated.clone()
        } else {
            materials.node_base.clone()
        };
    }

    // Update edges
    for (active, mut material) in &mut edges {
        material.0 = if active.0 {
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
fn init_materials(
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

// fn spawn_nodes(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<ColorMaterial>>,
//     tree: Res<PassiveTreeWrapper>,
//     scaling: Res<NodeScaling>,
//     config: Res<UserConfig>,
// ) {
//     let node_radius = scaling.base_radius;

//     // Create activated materials resource
//     commands.insert_resource(ActivatedMaterials {
//         node: materials.add(parse_hex_color(&config.colors["activated_nodes"])),
//         edge: materials.add(parse_hex_color(&config.colors["activated_edges"])),
//     });

//     let base_node_matl = materials.add(parse_hex_color(&config.colors["all_nodes"]));

//     for (_, node) in tree.tree.nodes.iter() {
//         let group = tree.tree.groups.get(&node.parent).unwrap();
//         let (x, y) = calculate_world_position(group, node.radius, node.position);

//         commands.spawn((
//             Mesh2d(meshes.add(Circle::new(node_radius))),
//             MeshMaterial2d(base_node_matl.clone()),
//             Transform::from_translation(Vec3::new(x, y, 0.0)),
//             NodeMarker(node.node_id),
//             NodeActive(false),
//         ));
//     }
// }

// fn spawn_edges(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<ColorMaterial>>,
//     tree: Res<PassiveTreeWrapper>,
//     config: Res<UserConfig>,
// ) {
//     let edge_color = materials.add(parse_hex_color(&config.colors["all_nodes"]));

//     tree.tree.edges.iter().for_each(|edge| {
//         let (start_node, end_node) = (
//             tree.tree.nodes.get(&edge.start).unwrap(),
//             tree.tree.nodes.get(&edge.end).unwrap(),
//         );

//         let (start_group, end_group) = (
//             tree.tree.groups.get(&start_node.parent).unwrap(),
//             tree.tree.groups.get(&end_node.parent).unwrap(),
//         );

//         let start_pos =
//             calculate_world_position(start_group, start_node.radius, start_node.position);
//         let end_pos = calculate_world_position(end_group, end_node.radius, end_node.position);
//         let start = Vec2::new(start_pos.0, start_pos.1);
//         let end = Vec2::new(end_pos.0, end_pos.1);

//         let delta = end - start;
//         let width = delta.length();
//         let height = 20.0;
//         let angle = delta.y.atan2(delta.x);
//         let midpoint = start.lerp(end, 0.5);

//         commands.spawn((
//             Mesh2d(meshes.add(Rectangle::new(width, height))),
//             MeshMaterial2d(edge_color.clone()),
//             EdgeMarker((edge.start, edge.end)),
//             Transform::from_translation(midpoint.extend(-0.01)) // move slightly backward so it is behind a node.
//                 .with_rotation(Quat::from_rotation_z(angle)),
//             EdgeActive(false),
//         ));
//     });
// }
