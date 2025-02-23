use std::{
    boxed::Box,
    ops::ControlFlow,
    sync::{atomic::AtomicBool, atomic::Ordering, Arc, Mutex},
    time::Duration,
};

use bevy::prelude::Color;
use bevy::{
    color::palettes::tailwind,
    prelude::{Visibility, *},
    render::{mesh::ConvexPolygonMeshBuilder, render_graph::Edge},
    text::CosmicBuffer,
    time::common_conditions::on_timer,
    utils::hashbrown::HashSet,
};
use poe_tree::{
    consts::{get_char_starts_node_map, CHAR_START_NODES, LEVEL_ONE_NODES},
    stats::Stat,
    type_wrappings::{EdgeId, NodeId},
    PassiveTree,
};

use crate::{
    components::*,
    consts::{DEFAULT_SAVE_PATH, SEARCH_THRESHOLD},
    events::{self, ManualNodeHighlightWithColour, NodeActivationReq, *},
    materials::{self, GameMaterials},
    mouse::handle_node_clicks,
    resources::*,
    search, PassiveTreeWrapper,
};

pub fn clear(
    nodes: Query<(Entity, &NodeMarker)>,
    edges: Query<(Entity, &EdgeMarker)>,

    mut commands: Commands,
    mut active_character: ResMut<ActiveCharacter>,

    mut node_deactivation_tx: EventWriter<NodeDeactivationReq>,
    mut edge_deactivation_tx: EventWriter<EdgeDeactivationReq>,
    mut path_needs_repair: ResMut<PathRepairRequired>,
    mut selected_history: ResMut<MouseSelecetedNodeHistory>,
) {
    path_needs_repair.set_unrequired();
    selected_history.clear();
    log::debug!("Clear command received.");
    active_character.activated_node_ids.clear();
    assert_eq!(
        0,
        active_character.activated_node_ids.len(),
        "active character's activated node count should be 0"
    );

    // FIXME: Technically the remove::<Node/EdgeActive>() don't need to be here as they're supposedly taken care of by the events, but
    // in practie i've noticed paths not being cleaned up and as scheduling is tedious... fukit.
    nodes
        .iter()
        .filter(|(_ent, nid)| nid.0 != active_character.starting_node)
        .for_each(|(ent, nid)| {
            commands.entity(ent).remove::<ManuallyHighlighted>();
            commands.entity(ent).remove::<VirtualPathMember>();
            // commands.entity(ent).remove::<NodeActive>();

            node_deactivation_tx.send(NodeDeactivationReq(**nid));
        });

    edges.iter().for_each(|(ent, eid)| {
        commands.entity(ent).remove::<ManuallyHighlighted>();
        commands.entity(ent).remove::<VirtualPathMember>();
        // commands.entity(ent).remove::<EdgeActive>();
        let (start, end) = eid.as_tuple();
        edge_deactivation_tx.send(EdgeDeactivationReq(start, end));
    });

    assert_eq!(
        0,
        active_character.activated_node_ids.len(),
        "active character's activated node count should be 0"
    );

    log::debug!("ClearAll executed successfully, NOTHING should be highlighted/coloured etc.");
}

pub fn path_repair(
    tree: Res<PassiveTreeWrapper>,
    recently_selected: Res<MouseSelecetedNodeHistory>,
    query: Query<&NodeMarker, With<NodeActive>>,
    root_node: Res<RootNode>,
    mut activator: EventWriter<NodeActivationReq>,
    mut sync_char: EventWriter<SyncCharacterReq>,
    mut path_needs_repair: ResMut<PathRepairRequired>,
) {
    // the most likely reason for path repair is a mouse activity breaking a path.
    let Some(most_recent) = recently_selected.back() else {
        path_needs_repair.set_unrequired();
        return;
    };

    let root_node = root_node.0.expect("Protected by run conditions");

    let active_nodes = query
        .into_iter()
        // A user selecting a node wayyyyy off will have marked it active.
        // So we strip out there most recent cursor selection and the root.
        .filter(|nid| nid.0 != *most_recent)
        .map(|n| **n)
        .collect::<Vec<NodeId>>();

    log::debug!(
        "Attempting path repair from {} to any of {:#?}",
        &most_recent,
        &active_nodes
    );

    let shortest_path = tree
        .shortest_to_target_from_any_of(*most_recent, &active_nodes)
        .unwrap_or_default();

    match shortest_path.is_empty() {
        false => {
            log::debug!(
                "Found a path between {most_recent}, and target {:#?}",
                shortest_path
            );
            if shortest_path.len() > PassiveTree::STEP_LIMIT as usize {
                log::warn!("User is attemtping to allocate points a total of more than {} points! which isn't allowed!",
                PassiveTree::STEP_LIMIT
            );
                //TODO: Throw warning text event
                return;
            };
            shortest_path.iter().for_each(|nid| {
                activator.send(NodeActivationReq(*nid));
            });
            path_needs_repair.set_unrequired();
            sync_char.send(SyncCharacterReq());
            log::debug!(
                "{} Activation reqs sent, char sync requested",
                shortest_path.len()
            );
        }
        true => {
            log::warn!("Unable to find a path from the {} to the any of the {} nodes in active_nodes, so instead we're trying to the root_node",
            &most_recent,
            &active_nodes.len()
        );
            let shortest_path = tree.bfs_any(root_node, &active_nodes);
            assert!(
                !shortest_path.is_empty(),
                "It should be impossible to return bfs_any without being able to reach the root_node"
            );

            shortest_path.into_iter().for_each(|nid| {
                activator.send(NodeActivationReq(nid));
            });
            path_needs_repair.request_path_repair();
        }
    }
}
