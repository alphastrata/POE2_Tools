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

fn populate_optimiser(
    mut optimiser: ResMut<Optimiser>,
    tree: Res<PassiveTreeWrapper>,
    active_character: Res<ActiveCharacter>,
    mut req: EventReader<OptimiseReq>,
) {
    log::trace!("Optimise requested");
    let baseline = active_character
        .activated_node_ids
        .iter()
        .map(|v| *v)
        .collect::<Vec<NodeId>>();

    req.read().for_each(|req| {
        if optimiser.is_available() {
            optimiser.set_busy();
            optimiser.results = tree
                .branches(&active_character.activated_node_ids)
                .iter()
                .flat_map(|opt| tree.take_while_better(*opt, &req.selector, req.delta, &baseline))
                .collect();
        }
        optimiser.set_available();
    })
}
