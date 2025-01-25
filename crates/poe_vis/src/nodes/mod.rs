use bevy::{prelude::*, utils::HashSet};
use materials::GameMaterials;
use poe_tree::{calculate_world_position, PassiveTree};

use crate::components::{
    EdgeActive, EdgeInactive, EdgeMarker, NodeActive, NodeInactive, NodeMarker,
};

pub mod hover;
pub mod materials;

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
    pub hover_multiplier: f32,
    pub hover_fade_time: f32,
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

        for mut transform in &mut node_query {
            transform.scale = Vec3::splat(final_scale);
        }
    }
}

pub fn spawn_nodes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<materials::GameMaterials>,
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
            Transform::from_translation(Vec3::new(x, -y, 0.0)),
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
        let start = Vec2::new(start_pos.0, -start_pos.1);
        let end = Vec2::new(end_pos.0, -end_pos.1);

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

    let to_add = &character.character.activated_node_ids;

    // Find and activate the starting node, and any additional nodes from the character.
    for (entity, marker) in node_query.iter() {
// If we found a starting_node, chances are there's more.

        if marker.0 == character.character.starting_node || to_add.contains(&marker.0){
            commands
                .entity(entity)
                .remove::<NodeInactive>()
                .insert(NodeActive);

        }
    }
}
/// Updates inactive/active nodes' materials and size etc.
pub fn update_nodes(
    materials: Res<materials::GameMaterials>,
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
pub fn update_edges(
    mut commands: Commands,
    edge_query: Query<(Entity, &EdgeMarker), With<EdgeInactive>>,
    node_query: Query<(&NodeMarker, Option<&NodeActive>)>,
) {
    let active_nodes: HashSet<poe_tree::type_wrappings::NodeId> =
        node_query.into_iter().map(|(n, _)| n.0).collect();

    for (entity, marker) in edge_query.iter() {
        let start_active = active_nodes.contains(&marker.0.0);
        let end_active = active_nodes.contains(&marker.0.1);

        match (start_active, end_active) {
            (true, true) => {
                commands
                    .entity(entity)
                    .remove::<EdgeInactive>()
                    .insert(EdgeActive);
            }

            _ => {
                commands
                    .entity(entity)
                    .remove::<EdgeActive>()
                    .insert(EdgeInactive);
            }
        }
    }
}
