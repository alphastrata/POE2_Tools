mod edges;
mod generated;
mod misc;
mod nodes;
mod optimiser_utils;
mod paths;
mod virtual_paths;

pub use edges::*;
pub use generated::{parse_tailwind_color, tailwind_to_egui};
pub use misc::*;
pub use nodes::*;
pub use optimiser_utils::*;
pub use paths::*;

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

pub(crate) struct BGServicesPlugin;

impl Plugin for BGServicesPlugin {
    fn build(&self, app: &mut App) {
        app
            // Spacing..
            .add_event::<ClearAll>()
            .add_event::<ClearSearchResults>()
            .add_event::<ClearVirtualPath>()
            .add_event::<DrawCircleReq>()
            .add_event::<DrawRectangleReq>()
            .add_event::<EdgeActivationReq>()
            .add_event::<EdgeColourReq>()
            .add_event::<EdgeDeactivationReq>()
            .add_event::<LoadCharacterReq>()
            .add_event::<ManualEdgeHighlightWithColour>()
            .add_event::<ManualNodeHighlightWithColour>()
            .add_event::<MoveCameraReq>()
            .add_event::<NodeActivationReq>()
            .add_event::<NodeColourReq>()
            .add_event::<NodeDeactivationReq>()
            .add_event::<NodeDeactivationReq>()
            .add_event::<NodeScaleReq>()
            .add_event::<OptimiseReq>()
            .add_event::<OverrideCharacterNodesReq>()
            .add_event::<SaveCharacterAsReq>()
            .add_event::<SaveCharacterReq>()
            .add_event::<ShowSearch>()
            .add_event::<SyncCharacterReq>()
            .add_event::<ThrowWarning>()
            .add_event::<VirtualPathReq>()
            //spacing..
            ;

        app //
            .init_resource::<Toggles>()
            .insert_resource(Optimiser {
                results: Vec::new(),
                status: JobStatus::Available,
            })
            .insert_resource(PathRepairRequired(false));

        app.add_systems(
            PostUpdate,
            ((
                // sync_active_with_character.run_if(active_nodes_changed),
                /* Only scan for edges when we KNOW the path is valid */
                scan_edges_for_active_updates.run_if(resource_equals(PathRepairRequired(false))),
                //deactivations:
                process_node_deactivations.run_if(on_event::<NodeDeactivationReq>),
                process_edge_deactivations,
                scan_edges_for_inactive_updates,
                /* happening all the time with camera moves. */
                adjust_node_sizes,
            )
                .after(clear),),
        );
        app.add_systems(
            Update,
            (
                //
                //activations:
                process_node_activations.run_if(on_event::<NodeActivationReq>),
                process_edge_activations,
                // Lock the rate we populate the virtual paths
                populate_virtual_path.run_if(on_event::<VirtualPathReq>.and(time_passed(0.080))),
                process_virtual_paths.after(populate_virtual_path),
                clear_virtual_paths.run_if(
                    on_event::<ClearVirtualPath>
                        .or(on_event::<ClearAll>.or(CameraSettings::is_moving)),
                ),
                process_manual_node_highlights.run_if(on_event::<ManualNodeHighlightWithColour>),
                process_manual_edge_highlights.run_if(on_event::<ManualEdgeHighlightWithColour>),
                /* Pretty lightweight, can be spammed.*/
                process_node_colour_changes.run_if(on_event::<NodeColourReq>),
                process_edge_colour_changes.run_if(on_event::<EdgeColourReq>),
                /* Runs a BFS so, try not to spam it.*/
                path_repair
                    .run_if(resource_exists::<RootNode>)
                    .run_if(sufficient_active_nodes)
                    .run_if(
                        resource_equals(PathRepairRequired(true))
                            .or(resource_changed::<ActiveCharacter>),
                    ),
            ),
        );

        // Optimiser routines:
        app.add_systems(
            PostUpdate,
            //TODO: cap the framerate that this can run at...i.e && NOT WORKING
            populate_optimiser.run_if(on_event::<OptimiseReq>),
        );

        app.add_systems(Update, clear.run_if(on_event::<ClearAll>));

        log::debug!("BGServices plugin enabled");
    }
}

// Colours & Aesthetics.
fn process_node_colour_changes(
    mut colour_events: EventReader<NodeColourReq>,
    mut materials_q: Query<&mut MeshMaterial2d<ColorMaterial>>,
) {
    colour_events.read().for_each(|NodeColourReq(entity, mat)| {
        if let Ok(mut m) = materials_q.get_mut(*entity) {
            m.0 = mat.clone_weak();
        }
    });
}
fn process_edge_colour_changes(
    mut colour_events: EventReader<EdgeColourReq>,
    mut materials_q: Query<&mut MeshMaterial2d<ColorMaterial>>,
) {
    colour_events.read().for_each(|EdgeColourReq(entity, mat)| {
        if let Ok(mut m) = materials_q.get_mut(*entity) {
            m.0 = mat.clone_weak();
        }
    });
}
fn path_repair(
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
fn process_manual_edge_highlights(
    mut events: EventReader<ManualEdgeHighlightWithColour>,
    mut colour_events: EventWriter<EdgeColourReq>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut game_materials: ResMut<GameMaterials>,
    query: Query<(Entity, &EdgeMarker)>,
) {
    events.read().for_each(
        |ManualEdgeHighlightWithColour(req_start, req_end, colour_str)| {
            let mat = game_materials
                .other
                .entry(colour_str.to_owned())
                .or_insert_with(|| {
                    let color = generated::parse_tailwind_color(colour_str);
                    materials.add(color)
                })
                .clone();

            query.iter().for_each(|(ent, marker)| {
                let (e_start, e_end) = marker.as_tuple();
                let mut go = false;
                match (e_start == *req_start, e_end == *req_end) {
                    (true, true) => go = true,
                    _ => {
                        // either way is a match...
                        //TODO: we should try to make this more ergonomic...
                        if e_start == *req_end && e_end == *req_start {
                            go = true;
                        }
                    }
                }
                if go {
                    log::info!("manual edge highlight action.");
                    commands.entity(ent).remove::<EdgeInactive>();
                    commands.entity(ent).remove::<EdgeActive>();
                    colour_events.send(EdgeColourReq(ent, mat.clone_weak()));
                    commands.entity(ent).insert(ManuallyHighlighted);
                    log::info!("manual edge highlight complete.");
                }
            });
        },
    );
}

fn process_manual_node_highlights(
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
