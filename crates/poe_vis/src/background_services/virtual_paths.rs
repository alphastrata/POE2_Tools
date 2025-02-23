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

pub fn something_is_hovered(query: Query<&NodeMarker, With<Hovered>>) -> bool {
    // println!("SOMETHING HOVERED");
    !query.is_empty()
}
pub fn nothing_is_hovered(query: Query<&NodeMarker, With<Hovered>>) -> bool {
    // println!("NOTHING HOVERED");
    query.is_empty()
}

pub(crate) fn process_virtual_paths(
    mut colour_events: EventWriter<NodeColourReq>,
    game_materials: Res<GameMaterials>,
    edges: Query<(Entity, &EdgeMarker), (With<VirtualPathMember>, Without<EdgeActive>)>,
    nodes: Query<(Entity, &NodeMarker), (With<VirtualPathMember>, Without<NodeActive>)>,
) {
    nodes.iter().for_each(|(ent, _em)| {
        let mat = game_materials.blue.clone_weak();
        colour_events.send(NodeColourReq(ent, mat.clone_weak()));
    });

    edges.iter().for_each(|(ent, _em)| {
        let mat = game_materials.blue.clone_weak();
        colour_events.send(NodeColourReq(ent, mat.clone_weak()));
    });
}
pub(crate) fn populate_virtual_path(
    mut commands: Commands,
    tree: Res<PassiveTreeWrapper>,
    active_character: Res<ActiveCharacter>,
    mut virt_path_req: EventReader<VirtualPathReq>,
    edges: Query<(Entity, &EdgeMarker), Without<EdgeActive>>,
    nodes: Query<(Entity, &NodeMarker), Without<NodeActive>>,
) {
    let hover_hits: Vec<NodeId> = virt_path_req.read().map(|req| **req).collect();
    let best = active_character
        .activated_node_ids
        .iter()
        .filter_map(|&candidate| tree.shortest_to_target_from_any_of(candidate, &hover_hits))
        .min_by_key(|path| path.len());

    if best.is_none() {
        log::warn!(
            "No best path found from {:#?} to any of {:#?}",
            hover_hits,
            active_character.activated_node_ids
        );
        return;
    }

    let best = best.unwrap();
    nodes
        .iter()
        .filter(|(_, nm)| best.contains(&nm.0))
        .for_each(|(ent, _)| {
            commands.entity(ent).insert(VirtualPathMember);
        });

    let m_cmd = Arc::new(Mutex::new(&mut commands));
    edges.par_iter().for_each(|(ent, em)| {
        let (start, end) = em.as_tuple();
        if best.contains(&start) && best.contains(&end) {
            m_cmd.lock().unwrap().entity(ent).insert(VirtualPathMember);
        }
    });
}

pub(crate) fn clear_virtual_paths(
    mut commands: Commands,
    mut colour_nodes: EventWriter<NodeColourReq>,
    mut colour_events: EventWriter<EdgeColourReq>,
    game_materials: Res<GameMaterials>,
    edges: Query<(Entity, &EdgeMarker), (With<VirtualPathMember>, Without<EdgeActive>)>,
    nodes: Query<(Entity, &NodeMarker), (With<VirtualPathMember>, Without<NodeActive>)>,
) {
    nodes.iter().for_each(|(ent, _em)| {
        let mat = game_materials.node_base.clone_weak();
        commands.entity(ent).remove::<VirtualPathMember>();

        colour_nodes.send(NodeColourReq(ent, mat.clone_weak()));
    });

    edges.iter().for_each(|(ent, _em)| {
        let mat = game_materials.edge_base.clone_weak();
        commands.entity(ent).remove::<VirtualPathMember>();

        colour_events.send(EdgeColourReq(ent, mat.clone_weak()));
    });
}
