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

use super::generated;

//Activations
pub(crate) fn process_node_activations(
    mut activation_events: EventReader<NodeActivationReq>,
    mut colour_events: EventWriter<NodeColourReq>,
    query: Query<(Entity, &NodeMarker), (With<NodeInactive>, Without<ManuallyHighlighted>)>,
    mut commands: Commands,
    root_node: Res<RootNode>,
    game_materials: Res<GameMaterials>,
) {
    let events: Vec<NodeId> = activation_events.read().map(|nar| nar.0).collect();

    let mat = &game_materials.node_activated;
    let activations = query
        .iter()
        .map(|(ent, nid)| {
            if events.contains(nid) || nid.0 == root_node.0.unwrap_or_default() {
                commands.entity(ent).remove::<NodeInactive>();
                log::trace!("Activating Node {}", **nid);
                commands.entity(ent).insert(NodeActive);
                colour_events.send(NodeColourReq(ent, mat.clone_weak()));
                log::trace!("Colour change requested for  Node {}", **nid);
            }
        })
        .count();
    log::debug!("{activations} activation events processed.");
}
// Colours & Aesthetics.
pub(crate) fn process_node_colour_changes(
    mut colour_events: EventReader<NodeColourReq>,
    mut materials_q: Query<&mut MeshMaterial2d<ColorMaterial>>,
) {
    colour_events.read().for_each(|NodeColourReq(entity, mat)| {
        if let Ok(mut m) = materials_q.get_mut(*entity) {
            m.0 = mat.clone_weak();
        }
    });
}
pub(crate) fn process_manual_node_highlights(
    mut events: EventReader<ManualNodeHighlightWithColour>,
    mut colour_events: EventWriter<NodeColourReq>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut game_materials: ResMut<GameMaterials>,
    query: Query<(Entity, &NodeMarker)>,
) {
    events
        .read()
        .for_each(|ManualNodeHighlightWithColour(node_id, colour_str)| {
            let mat = game_materials
                .other
                .entry(colour_str.to_owned())
                .or_insert_with(|| {
                    let color = generated::parse_tailwind_color(colour_str);
                    materials.add(color)
                })
                .clone();

            query.iter().for_each(|(ent, marker)| {
                if **marker == *node_id {
                    commands.entity(ent).remove::<NodeInactive>();
                    commands.entity(ent).remove::<NodeActive>();
                    colour_events.send(NodeColourReq(ent, mat.clone_weak()));
                    commands.entity(ent).insert(ManuallyHighlighted);
                }
            });
        });
}

// Deactivations
pub(crate) fn process_node_deactivations(
    mut deactivation_events: EventReader<NodeDeactivationReq>,
    mut colour_events: EventWriter<NodeColourReq>,
    query: Query<(Entity, &NodeMarker), (With<NodeActive>, Without<ManuallyHighlighted>)>,
    mut commands: Commands,
    game_materials: Res<GameMaterials>,
) {
    let events: Vec<NodeId> = deactivation_events.read().map(|ndr| ndr.0).collect();

    let mat = &game_materials.node_base;
    query
        .iter()
        // we never want to deactivate start/root nodes.
        .filter(|(_ent, nid)| !CHAR_START_NODES.iter().any(|v| *v == ***nid))
        .for_each(|(ent, nid)| {
            if events.contains(nid) {
                commands.entity(ent).remove::<NodeActive>();
                log::trace!("Deactivating Node {}", **nid);
                commands.entity(ent).insert(NodeInactive);
                colour_events.send(NodeColourReq(ent, mat.clone_weak()));
                log::trace!("Colour reset requested for Node {}", **nid);
            }
        })
}

pub(crate) fn active_nodes_changed(query: Query<(), Changed<NodeActive>>) -> bool {
    !query.is_empty()
}

pub(crate) fn active_edges_changed(query: Query<(), Changed<EdgeActive>>) -> bool {
    !query.is_empty()
}

pub(crate) fn sufficient_active_nodes(query: Query<&NodeMarker, With<NodeActive>>) -> bool {
    query.iter().count() > 1 // Only run if at least 2 nodes are active
}
