use std::ops::ControlFlow;
use std::sync::{Arc, Mutex};

use bevy::prelude::Visibility;
use bevy::{
    prelude::*,
    render::{mesh::ConvexPolygonMeshBuilder, render_graph::Edge},
    utils::hashbrown::HashSet,
};
use bevy_cosmic_edit::{CosmicEditBuffer, CosmicTextChanged};
use poe_tree::type_wrappings::{EdgeId, NodeId};

use crate::consts::SEARCH_THRESHOLD;
use crate::{
    components::*,
    events::{self, NodeActivationReq, *},
    materials::GameMaterials,
    mouse::handle_node_clicks,
    resources::*,
    PassiveTreeWrapper,
};
use crate::{materials, search};

pub(crate) struct BGServicesPlugin;

impl Plugin for BGServicesPlugin {
    fn build(&self, app: &mut App) {
        app
            // Spacing..
            .add_event::<EdgeActivationReq>()
            .add_event::<EdgeDeactivationReq>()
            .add_event::<EdgeColourReq>()
            .add_event::<NodeActivationReq>()
            .add_event::<NodeColourReq>()
            .add_event::<NodeScaleReq>()
            .add_event::<NodeDeactivationReq>()
            .add_event::<NodeDeactivationReq>()
            .add_event::<LoadCharacterReq>()
            .add_event::<SaveCharacterReq>()
            .add_event::<MoveCameraReq>()
            .add_event::<ShowSearch>()

            //spacing..
            ;

        app.insert_resource(PathRepairRequired(false));

        app.add_systems(
            Update,
            // TODO: rate-limiting
            (
                /* Users need to see paths magically illuminate */
                //activations:
                process_node_activations,
                process_edge_activations,
                /* Only scan for edges when we KNOW the path is valid */
                scan_edges_for_active_updates.run_if(resource_equals(PathRepairRequired(false))),
                //deactivations:
                process_node_deactivations,
                process_edge_deactivations,
                scan_edges_for_inactive_updates,
                /* happening all the time with camera moves. */
                adjust_node_sizes,
                /* Pretty lightweight, can be spammed.*/
                process_node_colour_changes,
                process_edge_colour_changes,
                /* Runs a BFS so, try not to spam it.*/
                validate_paths_between_active_nodes
                    .run_if(sufficient_active_nodes)
                    .run_if(resource_equals(PathRepairRequired(true))),
                /* Search */
                process_searchbox_visibility_toggle.run_if(on_event::<ShowSearch>),
                (
                    read_searchtext,
                    search_nodes_for,
                    process_search_results.run_if(resource_changed::<SearchState>),
                )
                    .chain(),
            ),
        );
        log::debug!("BGServices plugin enabled");
    }
}

// Conditional Helpers to rate-limit systems:
fn active_nodes_changed(query: Query<(), Changed<NodeActive>>) -> bool {
    !query.is_empty()
}

fn active_edges_changed(query: Query<(), Changed<EdgeActive>>) -> bool {
    !query.is_empty()
}

fn sufficient_active_nodes(query: Query<&NodeMarker, With<NodeActive>>) -> bool {
    query.iter().count() >= 2 // Only run if at least 2 nodes are active
}

// BG SERVICES INBOUND:
fn process_scale_requests(
    mut scale_events: EventReader<NodeScaleReq>,
    mut transforms: Query<&mut Transform>,
) {
    scale_events
        .read()
        .for_each(|NodeScaleReq(entity, new_scale)| {
            if let Ok(mut t) = transforms.get_mut(*entity) {
                t.scale = Vec3::splat(*new_scale);
            }
        });
}

fn process_searchbox_visibility_toggle(
    mut commands: Commands,
    mut searchbox_query: Query<Entity, With<SearchMarker>>,
    mut searchbox_state: ResMut<SearchState>,
) {
    let Ok(sb) = searchbox_query.get_single_mut() else {
        log::warn!("Unable to get searchbox...");
        return;
    };

    searchbox_state.open = !searchbox_state.open;
    match searchbox_state.open {
        true => {
            commands.entity(sb).remove::<Visibility>();
            commands.entity(sb).insert(Visibility::Visible);
        }
        false => {
            commands.entity(sb).remove::<Visibility>();
            commands.entity(sb).insert(Visibility::Hidden);
        }
    }
}
fn read_searchtext(
    mut txt: EventReader<CosmicTextChanged>,
    mut searchbox_state: ResMut<SearchState>,
) {
    txt.read().for_each(|ctx| {
        let val = &ctx.0 .1;

        log::debug!("Search query is below the SEARCH_THRESHOLD");
        log::debug!("{}", val.len());
        log::debug!("{}", val);
        searchbox_state.search_query.push_str(val);
        return;
    });
}
fn search_nodes_for(tree: Res<PassiveTreeWrapper>, mut searchbox_state: ResMut<SearchState>) {
    tree.fuzzy_search_nodes(&searchbox_state.search_query)
        .into_iter()
        .for_each(|n| {
            log::debug!("{:#?}", &n);
            searchbox_state.search_results.insert(n);
        })
}

fn process_search_results(
    searchbox_state: Res<SearchState>,
    mut colour_events: EventWriter<NodeColourReq>,
    query: Query<(Entity, &NodeMarker)>,
    materials: Res<GameMaterials>,
) {
    let tx = Arc::new(Mutex::new(&mut colour_events));
    query.par_iter().for_each(|(ent, nm)| {
        if searchbox_state.search_results.contains(&(**nm)) {
            match tx.lock() {
                Ok(mut tx) => {
                    tx.send(NodeColourReq(ent, materials.purple.clone()));
                }
                Err(e) => {
                    log::error!("{}", e);
                }
            }
        }
    });
    dbg!(&searchbox_state.search_query);
}

//Activations
fn process_node_activations(
    mut activation_events: EventReader<NodeActivationReq>,
    mut colour_events: EventWriter<NodeColourReq>,
    query: Query<(Entity, &NodeMarker), With<NodeInactive>>,
    mut commands: Commands,
    game_materials: Res<GameMaterials>,
) {
    let events: Vec<NodeId> = activation_events.read().map(|nar| nar.0).collect();

    let mat = &game_materials.node_activated;
    query.iter().for_each(|(ent, nid)| {
        if events.contains(nid) {
            commands.entity(ent).remove::<NodeInactive>();
            log::trace!("Activating Node {}", **nid);
            commands.entity(ent).insert(NodeActive);
            colour_events.send(NodeColourReq(ent, mat.clone_weak()));
            log::trace!("Colour change requested for  Node {}", **nid);
        }
    })
}

fn process_edge_activations(
    mut activation_events: EventReader<EdgeActivationReq>,
    mut colour_events: EventWriter<EdgeColourReq>,
    query: Query<(Entity, &EdgeMarker), With<EdgeInactive>>,
    active_nodes: Query<&NodeMarker, With<NodeActive>>,
    mut commands: Commands,
    game_materials: Res<GameMaterials>,
) {
    //NOTE: the maximum size of an active_nodes set is (123 * 4 bytes)
    let active_nodes: HashSet<NodeId> = active_nodes.into_iter().map(|nid| nid.0).collect();

    //NOTE: these should always be pretty tiny, it'd realistically be the number of edges we could receive in a single frame.
    let requested: HashSet<(EdgeId, EdgeId)> = activation_events
        .read()
        .map(move |ear| ear.as_tuple())
        .collect();

    let mat = &game_materials.edge_activated;

    query
        .into_iter()
        .map(|(ent, edge_marker)| (ent, edge_marker.as_tuple()))
        .filter(|(_ent, edge)| requested.contains(edge))
        .filter(|(_ent, (start, end))| active_nodes.contains(start) && active_nodes.contains(end))
        .for_each(|(ent, (start, end))| {
            log::trace!("Activating Edge {start}..{end}");
            commands.entity(ent).remove::<EdgeInactive>();
            commands.entity(ent).insert(EdgeActive);
            colour_events.send(EdgeColourReq(ent, mat.clone_weak()));
            log::trace!("Colour change requested for Edge {start}..{end}");
        });
}

fn scan_edges_for_active_updates(
    mut edge_activator: EventWriter<EdgeActivationReq>,
    haystack: Query<&EdgeMarker, With<EdgeInactive>>,
    needles: Query<&NodeMarker, With<NodeActive>>,
) {
    let active_nodes: HashSet<NodeId> = needles.into_iter().map(|marker| **marker).collect();

    let mtx_edge_activator = std::sync::Arc::new(std::sync::Mutex::new(&mut edge_activator));
    // There are ~3200 edges (8bytes each), so even if we've activated half of them this usually shows performance benefits.
    // as we never really expect to be activating more than a handful of the nodes we do find, lock contention has been observed to be low,
    // relative to the searchspace.
    haystack.par_iter().for_each(|edg| {
        let (start, end) = edg.as_tuple();

        if active_nodes.contains(&start) && active_nodes.contains(&end) {
            match mtx_edge_activator.lock() {
                Ok(mut l_edge_activator) => {
                    l_edge_activator.send(EdgeActivationReq(start, end));
                }
                _ => {
                    log::error!("Unable to gain lock on the EventWriter to send an activation request for Edge {:?}", edg);
                },
            }
        }
    });
}

// Deactivations
fn process_node_deactivations(
    mut deactivation_events: EventReader<NodeDeactivationReq>,
    mut colour_events: EventWriter<NodeColourReq>,
    query: Query<(Entity, &NodeMarker), With<NodeActive>>,
    mut commands: Commands,
    game_materials: Res<GameMaterials>,
) {
    let events: Vec<NodeId> = deactivation_events.read().map(|ndr| ndr.0).collect();

    let mat = &game_materials.node_base;
    query.iter().for_each(|(ent, nid)| {
        if events.contains(nid) {
            commands.entity(ent).remove::<NodeActive>();
            log::trace!("Deactivating Node {}", **nid);
            commands.entity(ent).insert(NodeInactive);
            colour_events.send(NodeColourReq(ent, mat.clone_weak()));
            log::trace!("Colour reset requested for Node {}", **nid);
        }
    })
}

fn process_edge_deactivations(
    mut deactivation_events: EventReader<EdgeDeactivationReq>,
    mut colour_events: EventWriter<EdgeColourReq>,
    query: Query<(Entity, &EdgeMarker), With<EdgeActive>>,
    active_nodes: Query<&NodeMarker, With<NodeActive>>,
    mut commands: Commands,
    game_materials: Res<GameMaterials>,
) {
    let active_nodes: HashSet<NodeId> = active_nodes.into_iter().map(|nid| **nid).collect();

    let requested: HashSet<(EdgeId, EdgeId)> = deactivation_events
        .read()
        .map(|edr| edr.as_tuple())
        .collect();

    let mat = &game_materials.edge_base;

    query
        .into_iter()
        .map(|(ent, edge_marker)| (ent, edge_marker.as_tuple()))
        .filter(|(_ent, edge)| requested.contains(edge))
        .filter(|(_ent, (start, end))| !active_nodes.contains(start) || !active_nodes.contains(end))
        .for_each(|(ent, (start, end))| {
            log::trace!("Deactivating Edge {start}..{end}");
            commands.entity(ent).remove::<EdgeActive>();
            commands.entity(ent).insert(EdgeInactive);
            colour_events.send(EdgeColourReq(ent, mat.clone_weak()));
            log::trace!("Colour reset requested for Edge {start}..{end}");
        });
}

fn scan_edges_for_inactive_updates(
    mut edge_deactivator: EventWriter<EdgeDeactivationReq>,
    haystack: Query<&EdgeMarker, With<EdgeActive>>,
    needles: Query<&NodeMarker, With<NodeActive>>,
) {
    let active_nodes: HashSet<NodeId> = needles.into_iter().map(|marker| **marker).collect();

    // < 300 active edges are possible at any given time.
    haystack.iter().for_each(|edg| {
        let (start, end) = edg.as_tuple();
        if !active_nodes.contains(&start) || !active_nodes.contains(&end) {
            edge_deactivator.send(EdgeDeactivationReq(start, end));
        }
    });
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

/// Adjust each node’s Transform.scale so it doesn’t get too big or too small on screen.
fn adjust_node_sizes(
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

// PATHFINDING
fn validate_paths_between_active_nodes(
    tree: Res<PassiveTreeWrapper>,
    query: Query<&NodeMarker, With<NodeActive>>,
    mut activate_req: EventWriter<NodeActivationReq>,
    root_node: Res<RootNode>,
    path_needs_repair: ResMut<PathRepairRequired>,
) {
    let active_nodes: Vec<_> = query.iter().map(|m| m.0).collect();
    let active_and_validly_pathed =
        validate_path_to_root(tree, &active_nodes, &root_node, path_needs_repair);

    // let active_and_validly_pathed = tree.bfs_any(root_node.0.unwrap(), &active_nodes);

    // // When this panics we need to find the problematic nodes...
    // if !active_nodes
    //     .into_iter()
    //     .all(|v| active_and_validly_pathed.contains(&v))
    // {
    //     log::error!("Not all paths in the current active nodes can reach the root...");
    // }

    active_and_validly_pathed.into_iter().for_each(|an| {
        activate_req.send(NodeActivationReq(an));
    });
}

fn validate_path_to_root(
    tree: Res<PassiveTreeWrapper>,
    active_nodes: &[NodeId],
    root_node: &RootNode,
    mut path_needs_repair: ResMut<PathRepairRequired>,
) -> HashSet<NodeId> {
    if root_node.0.is_none() {
        log::warn!("Unable to begin pathfinding, we have no root node set.");
        return HashSet::new();
    }

    let mut seen: HashSet<NodeId> = HashSet::new();
    let start = root_node.0.unwrap();

    for &end in active_nodes.iter() {
        if seen.contains(&start) && seen.contains(&end) {
            log::debug!(
                "Skipping search for path [{}..{}] as it has already been checked.",
                start,
                end
            );
            continue;
        }

        let path = tree.bfs(start, end);

        // Insert all nodes from the path into seen
        for node in path.iter() {
            seen.insert(*node);
        }

        if path.len() > 1 {
            log::debug!(
                "Valid path found for [{}..{}], with {} steps",
                start,
                end,
                path.len()
            );
            break;
        }
    }

    if seen.is_empty() {
        // path_needs_repair.request_path_repair();
        //TODO: carry out path repair here!
    } else {
        path_needs_repair.set_unrequired();
    }

    seen
}
