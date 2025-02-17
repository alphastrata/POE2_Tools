mod generated;

pub use generated::{parse_tailwind_color, tailwind_to_egui};

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
    consts::LEVEL_ONE_NODES,
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

// Conditional Helpers to rate-limit systems:
pub fn active_nodes_changed(query: Query<(), Changed<NodeActive>>) -> bool {
    !query.is_empty()
}

pub fn active_edges_changed(query: Query<(), Changed<EdgeActive>>) -> bool {
    !query.is_empty()
}

pub fn sufficient_active_nodes(query: Query<&NodeMarker, With<NodeActive>>) -> bool {
    query.iter().count() > 1 // Only run if at least 2 nodes are active
}

pub fn something_is_hovered(query: Query<&NodeMarker, With<Hovered>>) -> bool {
    // println!("SOMETHING HOVERED");
    !query.is_empty()
}
pub fn nothing_is_hovered(query: Query<&NodeMarker, With<Hovered>>) -> bool {
    // println!("NOTHING HOVERED");
    query.is_empty()
}

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
            .add_event::<SaveCharacterAsReq>()
            .add_event::<SaveCharacterReq>()
            .add_event::<ShowSearch>()
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
            PreUpdate,
            ((
                sync_active_with_character.run_if(active_nodes_changed),
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
                process_load_character.run_if(on_event::<LoadCharacterReq>),
                process_save_character
                    .run_if(on_event::<SaveCharacterReq>.or(on_event::<SaveCharacterAsReq>)),
                /* Users need to see paths magically illuminate */
                //activations:
                process_node_activations.run_if(on_event::<NodeActivationReq>),
                process_edge_activations,
                // Lock the rate we populate the virtual paths at 50ms
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
                    .run_if(sufficient_active_nodes)
                    .run_if(resource_equals(PathRepairRequired(true))),
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
// SOURCE: https://github.com/bevyengine/bevy/blob/main/examples/ecs/run_conditions.rs
/// This is a function that returns a closure which can be used as a run condition.
///
/// This is useful because you can reuse the same run condition but with different variables.
/// This is how the common conditions module works.
fn time_passed(t: f32) -> impl FnMut(Local<f32>, Res<Time>) -> bool {
    move |mut timer: Local<f32>, time: Res<Time>| {
        // Tick the timer
        *timer += time.delta_secs();
        // Return true if the timer has passed the time
        *timer >= t
    }
}

fn sync_active_with_character(
    active_character: Res<ActiveCharacter>,
    mut deactivator: EventWriter<NodeDeactivationReq>,
    mut activator: EventWriter<NodeActivationReq>,
    query: Query<(Entity, &NodeMarker)>,
) {
    log::trace!("Updating character.");

    let active = Arc::new(Mutex::new(&mut activator));
    let deactive = Arc::new(Mutex::new(&mut deactivator));
    query.par_iter().for_each(|(_ent, nm)| {
        // We'll just be sloppy and potentially send activation requests to nodes that _may_
        // already BE active, and so on.
        match active_character.activated_node_ids.contains(&nm.0) {
            true => {
                active.lock().unwrap().send(NodeActivationReq(nm.0));
            }
            false => {
                deactive.lock().unwrap().send(NodeDeactivationReq(nm.0));
            }
        }
    });
}

pub fn clear(
    nodes: Query<(Entity, &NodeMarker)>,
    edges: Query<(Entity, &EdgeMarker)>,

    mut commands: Commands,
    mut active_character: ResMut<ActiveCharacter>,

    mut node_deactivation_tx: EventWriter<NodeDeactivationReq>,
    mut edge_deactivation_tx: EventWriter<EdgeDeactivationReq>,
) {
    log::debug!("Clear command received.");
    active_character.activated_node_ids.clear();
    assert_eq!(
        0,
        active_character.activated_node_ids.len(),
        "active character's activated node count should be 0"
    );

    nodes
        .iter()
        .filter(|(_ent, nid)| nid.0 != active_character.starting_node)
        .for_each(|(ent, nid)| {
            commands.entity(ent).remove::<ManuallyHighlighted>();
            commands.entity(ent).remove::<VirtualPathMember>();

            node_deactivation_tx.send(NodeDeactivationReq(**nid));
        });

    edges.iter().for_each(|(ent, eid)| {
        commands.entity(ent).remove::<ManuallyHighlighted>();
        commands.entity(ent).remove::<VirtualPathMember>();

        let (start, end) = eid.as_tuple();
        edge_deactivation_tx.send(EdgeDeactivationReq(start, end));
    });

    assert_eq!(
        0,
        active_character.activated_node_ids.len(),
        "active character's activated node count should be 0"
    );

    // JIC we always require path repair after a Clear
    log::debug!("ClearAll executed successfully, NOTHING should be highlighet/coloured etc.");
}

fn process_save_character(
    save: EventReader<SaveCharacterReq>,
    mut save_as: EventReader<SaveCharacterAsReq>, // "save as" event with a PathBuf
    mut last_save_loc: ResMut<LastSaveLocation>,
    mut active_character: ResMut<ActiveCharacter>,
    active_nodes: Query<&NodeMarker, With<NodeActive>>,
) {
    active_character.activated_node_ids = active_nodes.iter().map(|nm| **nm).collect();
    active_character.level = active_character.activated_node_ids.len() as u8;

    // Choose path based on events
    let path = if let Some(evt) = save_as.read().last() {
        // "Save as" event overrides last_save_loc
        **last_save_loc = (**evt).clone();
        (**evt).clone()
    } else if !save.is_empty() {
        last_save_loc.0.clone()
    } else {
        // Default fallback
        std::path::PathBuf::from(DEFAULT_SAVE_PATH)
    };

    if let Err(e) = active_character.save_to_toml(&path) {
        log::error!("{}", e);
    }
    log::debug!("Character Saved to {:?}", path);
}

fn process_load_character(
    mut loader: EventReader<LoadCharacterReq>,
    mut active_character: ResMut<ActiveCharacter>,
    mut root_node: ResMut<RootNode>,
) {
    println!("Load Character requested");
    loader.read().for_each(|req| {
        let path = &req.0;
        match path.extension().and_then(|s| s.to_str()) {
            Some("toml") => {
                // Use ours.
                match poe_tree::character::Character::load_from_toml(path) {
                    Some(character) => {
                        active_character.character = character;
                        println!("Load Character from OUR format finalised");
                    }
                    None => eprintln!("Failed to load TOML from {}", path.display()),
                }
            }
            Some("xml") => {
                // Assume XML is in PoB export format.
                match std::fs::read_to_string(path) {
                    Ok(xml_str) => {
                        match quick_xml::de::from_str::<poe_tree::pob_utils::POBCharacter>(&xml_str)
                        {
                            Ok(pob_char) => {
                                active_character.character = pob_char.into();
                                println!("Load Character from POB's format finalised");
                            }
                            Err(e) => log::error!("XML parse error in {}: {:?}", path.display(), e),
                        }
                    }
                    Err(e) => eprintln!("Failed to read {}: {:?}", path.display(), e),
                }
            }
            //TODO: throw UI error (There's an event for it ThrowWarning)
            Some(ext) => {
                log::error!("Unsupported file extension: {}", ext);
            }
            None => {
                log::error!("Could not determine file extension for {}", path.display());
            }
        }
    });

    LEVEL_ONE_NODES
        .iter()
        .flat_map(|v| active_character.activated_node_ids.get(v))
        .for_each(|v| {
            log::debug!("Resetting the root node to: {v}");

            **root_node = Some(*v)
        });
}

//Activations
fn process_node_activations(
    mut activation_events: EventReader<NodeActivationReq>,
    mut colour_events: EventWriter<NodeColourReq>,
    query: Query<(Entity, &NodeMarker), (With<NodeInactive>, Without<ManuallyHighlighted>)>,
    mut commands: Commands,
    root_node: Res<RootNode>,
    game_materials: Res<GameMaterials>,
) {
    let events: Vec<NodeId> = activation_events.read().map(|nar| nar.0).collect();

    let mat = &game_materials.node_activated;
    query.iter().for_each(|(ent, nid)| {
        if events.contains(nid) || nid.0 == root_node.0.unwrap_or_default() {
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
pub fn process_node_deactivations(
    mut deactivation_events: EventReader<NodeDeactivationReq>,
    mut colour_events: EventWriter<NodeColourReq>,
    query: Query<(Entity, &NodeMarker), (With<NodeActive>, Without<ManuallyHighlighted>)>,
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
    query: Query<(Entity, &EdgeMarker), (With<EdgeActive>, Without<ManuallyHighlighted>)>,
    active_nodes: Query<&NodeMarker, (With<NodeActive>, Without<ManuallyHighlighted>)>,
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
        node_query.iter_mut().for_each(|mut transform| {
            // Combine base scale with zoom scale.
            transform.scale = Vec3::splat(scaling.base_radius * clamped_scale);
        });
    }
}

fn path_repair(
    tree: Res<PassiveTreeWrapper>,
    recently_selected: Res<MouseSelecetedNodeHistory>,
    query: Query<&NodeMarker, With<NodeActive>>,
    root_node: Res<RootNode>,
    mut activator: EventWriter<NodeActivationReq>,
    mut path_needs_repair: ResMut<PathRepairRequired>,
) {
    // the most likely reason for path repair is a mouse activity breaking a path.
    let Some(most_recent) = recently_selected.back() else {
        unreachable!("Unreachable because we pre-set this value in a startup system.");
    };

    let root_node = root_node.0.unwrap_or_default(); // There is no NodeId == 0.
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

    let shortest_path = tree.bfs_any(*most_recent, &active_nodes);

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
            shortest_path.into_iter().for_each(|nid| {
                activator.send(NodeActivationReq(nid));
            });
            path_needs_repair.set_unrequired();
        }
        true => {
            log::warn!("Unable to find a path from the {} to the any of the {} nodes in active_nodes, so instead we're trying to the root_node",
            &most_recent,
            &active_nodes.len()
        );
            let shortest_path = tree.bfs_any(root_node, &active_nodes);
            assert!(
                !shortest_path.is_empty(),
                "It should be impossible to return bfs without being able to reach the root_node"
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

//TODO: bool flag
fn process_virtual_paths(
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
fn populate_virtual_path(
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

fn clear_virtual_paths(
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

// NOTE: atomics because I'm too lazy to think of a runtime condition etc for this ...
fn populate_optimiser(
    mut optimiser: ResMut<Optimiser>,
    tree: Res<PassiveTreeWrapper>,
    active_character: Res<ActiveCharacter>,
    mut req: EventReader<OptimiseReq>,
) {
    log::trace!("Optimise requested");
    req.read().for_each(|req| {
        if optimiser.is_available() {
            optimiser.set_busy();
            optimiser.results = tree
                .branches(&active_character.activated_node_ids)
                .iter()
                .flat_map(|opt| tree.take_while(*opt, &req.selector, req.delta))
                .collect();
        }
        optimiser.set_available();
    })
}
