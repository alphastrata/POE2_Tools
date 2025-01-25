//! This is where any systems that run automagically based on the state that a user manipulates data in the ECS to with their clicks.

use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use poe_tree::type_wrappings::NodeId;

use crate::{
    components::{EdgeActive, EdgeInactive, EdgeMarker, NodeActive, NodeInactive, NodeMarker},
    nodes::PassiveTreeWrapper,
};

pub fn node_active_changed(query: Query<(), Changed<NodeActive>>) -> bool {
    !query.is_empty()
}

pub fn edge_active_changed(query: Query<(), Changed<EdgeActive>>) -> bool {
    !query.is_empty()
}


pub fn sufficient_active_nodes(query: Query<&NodeMarker, With<NodeActive>>) -> bool {
    query.iter().count() >= 2 // Only run if at least 2 nodes are active
}


pub fn bg_edge_updater(
    node_query: Query<&NodeMarker, With<NodeActive>>,
    mut commands: Commands,
    edge_query: Query<(Entity, &EdgeMarker, Option<&EdgeActive>, Option<&EdgeInactive>)>,
) {
    // Get all active nodes
    let active_nodes: HashSet<NodeId> = node_query.iter().map(|m| m.0).collect();

    // Update edges
    for (edge_entity, edge_marker, is_active, is_inactive) in edge_query.iter() {
        let (start, end) = edge_marker.0;
        let should_be_active = active_nodes.contains(&start) && active_nodes.contains(&end);

        match (should_be_active, is_active.is_some(), is_inactive.is_some()) {
            (true, false, true) => {
                // Activate the edge
                commands
                    .entity(edge_entity)
                    .remove::<EdgeInactive>()
                    .insert(EdgeActive);
            }
            (false, true, false) => {
                // Deactivate the edge
                commands
                    .entity(edge_entity)
                    .remove::<EdgeActive>()
                    .insert(EdgeInactive);
            }
            _ => {
                // No state change needed
            }
        }

        // Debug logging
        #[cfg(debug_assertions)]
        if should_be_active {
            log::debug!("Edge activated between {} and {}", start, end);
        } else {
            log::debug!("Edge deactivated between {} and {}", start, end);
        }
    }
}

pub fn pathfinding_system(
    tree: Res<PassiveTreeWrapper>,
    root: Res<crate::config::RootNode>,
    character: Res<crate::config::ActiveCharacter>,
    active_nodes: Query<&NodeMarker, With<NodeActive>>,
    all_node_entities: Query<(Entity, &NodeMarker)>,
    mut commands: Commands,
) {
    let root_id = match root.0 {
        Some(id) => id,
        None => return,
    };

    let active_ids: HashSet<NodeId> = active_nodes.iter().map(|m| m.0).collect();
    let node_entity_map: HashMap<NodeId, Entity> =
        all_node_entities.iter().map(|(e, m)| (m.0, e)).collect();

    // Get newly activated nodes not in character data
    let new_activations: Vec<NodeId> = active_ids
        .iter()
        .filter(|id| !character.character.activated_node_ids.contains(id))
        .copied()
        .collect();

    for new_node in new_activations {
        let path = tree.tree.bfs(root_id, new_node);

        for node_id in path {
            if !active_ids.contains(&node_id) {
                if let Some(&entity) = node_entity_map.get(&node_id) {
                    commands
                        .entity(entity)
                        .remove::<NodeInactive>()
                        .insert(NodeActive);
                }
            }
        }
    }
}
