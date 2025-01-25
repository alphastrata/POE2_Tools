use bevy::{prelude::*, utils::HashSet};
use materials::GameMaterials;
use poe_tree::{calculate_world_position, PassiveTree};

use crate::components::{
    EdgeActive, EdgeInactive, EdgeMarker, NodeActive, NodeInactive, NodeMarker, materials::*
};

pub mod hover;





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

/// Adjust each node’s `Transform.scale` based on camera zoom and node scaling constraints.
pub fn adjust_node_sizes(
    camera_query: Query<&OrthographicProjection, With<Camera2d>>,
    scaling: Res<NodeScaling>,
    mut node_query: Query<&mut Transform, With<NodeMarker>>,
) {
    if let Ok(projection) = camera_query.get_single() {
        // Compute zoom-based scaling factor.
        let zoom_scale = 1.0 / projection.scale;
        // Clamp the zoom scale to avoid extreme values.
        let clamped_scale = zoom_scale.clamp(scaling.min_scale, scaling.max_scale);

        // Apply scale adjustment to all nodes.
        for mut transform in &mut node_query {
            // Combine base scale with zoom scale.
            transform.scale = Vec3::splat(scaling.base_radius * clamped_scale);
        }
    }
}

pub fn highlight_starting_node(
    character: Res<crate::config::ActiveCharacter>,
    mut commands: Commands,
    node_query: Query<(Entity, &NodeMarker), With<NodeInactive>>,
){
    for (entity, marker) in node_query.iter() {
        if marker.0 == character.character.starting_node
            || character.character.activated_node_ids.contains(&marker.0)
        {
            commands.entity(entity)
                .remove::<NodeInactive>()
                .insert(NodeActive); // no direct transform/material here
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
