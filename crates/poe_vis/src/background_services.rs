use bevy::{
    core_pipeline::oit::resolve::node,
    prelude::*,
    utils::{HashMap, HashSet},
};
use poe_tree::type_wrappings::NodeId;

use crate::{
    components::{EdgeActive, EdgeMarker, NodeActive, NodeMarker},
    nodes::PassiveTreeWrapper,
};

pub fn bg_edge_updater(
    node_query: Query<(&NodeMarker, &NodeActive)>,
    mut edge_query: Query<(&EdgeMarker, &mut EdgeActive)>,
) {
    // Get active nodes first
    let active_nodes: HashSet<NodeId> = node_query
        .iter()
        .filter(|(_, active)| active.0)
        .map(|(marker, _)| marker.0)
        .collect();

    // Update edges in a single pass
    for (edge_marker, mut active) in &mut edge_query {
        let (start, end) = edge_marker.0;
        active.0 = active_nodes.contains(&start) && active_nodes.contains(&end);

        #[cfg(debug_assertions)]
        {
            //TODO: remove me
            let initial = active.0;

            if initial {
                log::debug!("Edge was ACTIVE for {} to {}", start, end);
                log::debug!(
                    "flip == {}",
                    active_nodes.contains(&start) && active_nodes.contains(&end)
                );
            }
        }
    }
}

pub fn pathfinding_system(
    tree: Res<PassiveTreeWrapper>,
    active_nodes: Query<&NodeMarker, With<NodeActive>>,
    mut all_nodes: Query<(&NodeMarker, &mut NodeActive)>,
) {
    // Get all active node IDs
    let active_ids: HashSet<NodeId> = active_nodes.iter().map(|m| m.0).collect();
    let active_list: Vec<NodeId> = active_ids.iter().copied().collect();
    let mut activated = HashSet::new();

    // Check all pairs of active nodes
    (0..active_list.len()).for_each(|i| {
        ((i + 1)..active_list.len()).for_each(|j| {
            let a = active_list[i];
            let b = active_list[j];

            // Find path between these nodes in the full tree
            tree.tree.bfs(a, b).into_iter().for_each(|node_id| {
                // Only activate nodes that weren't already active
                if !active_ids.contains(&node_id) && !activated.contains(&node_id) {
                    // Update node component directly
                    if let Some((_, mut active)) =
                        all_nodes.iter_mut().find(|(marker, _)| marker.0 == node_id)
                    {
                        active.0 = true;
                        activated.insert(node_id);
                        log::debug!("Pathfinding Activated {}", node_id);
                    }
                }
                // NOTE:
                // We leave the edges to be picked up for activation by another bg system,
                // this is to simplify us having to manage them.
            });
        });
    });
}
