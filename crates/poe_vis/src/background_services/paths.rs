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
