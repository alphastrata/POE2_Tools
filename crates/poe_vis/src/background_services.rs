use bevy::{prelude::*, render::mesh::ConvexPolygonMeshBuilder, utils::hashbrown::HashSet};
use poe_tree::type_wrappings::{EdgeId, NodeId};

use crate::{
    components::*,
    events::{self, NodeActivationReq, *},
    materials::GameMaterials,
    mouse::handle_node_clicks,
    resources::*,
    PassiveTreeWrapper,
};

pub(crate) struct BGServicesPlugin;

impl Plugin for BGServicesPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<NodeScaleReq>()
            .add_event::<NodeColourReq>()
            .add_event::<NodeActivationReq>()
            .add_event::<EdgeActivationReq>();
        /* move this over*/

        app.add_systems(
            Update,
            // TODO: rate-limiting
            (
                /* Users need to see paths magically illuminate */
                process_node_activations,
                process_edge_activations,
                /* happening all the time with camera moves. */
                adjust_node_sizes,
                /* Pretty lightweight, can be spammed.*/
                process_node_colour_changes.run_if(active_nodes_changed),
                process_edge_colour_changes.run_if(active_edges_changed),
                /* Runs a BFS so, try not to spam it.*/
                validate_paths_between_active_nodes
                    .after(handle_node_clicks)
                    .run_if(sufficient_active_nodes),
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
            commands.entity(ent).insert(NodeActive);
            colour_events.send(NodeColourReq(ent, mat.clone_weak()));
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
        .for_each(|(ent, _)| {
            commands.entity(ent).remove::<EdgeInactive>();
            commands.entity(ent).insert(EdgeActive);
            colour_events.send(EdgeColourReq(ent, mat.clone_weak()));
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
/// Adjust each node’s Transform.scale based on camera zoom and node scaling constraints.
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
) {
    let active_nodes: Vec<_> = query.iter().map(|m| m.0).collect();
    let active_and_validly_pathed = find_all_paths(tree, &active_nodes);
    active_and_validly_pathed.into_iter().for_each(|an| {
        activate_req.send(NodeActivationReq(an));
    });
}

fn find_all_paths(tree: Res<'_, PassiveTreeWrapper>, active_nodes: &[NodeId]) -> HashSet<NodeId> {
    let mut seen: HashSet<NodeId> = HashSet::new();

    (0..active_nodes.len()).for_each(|i| {
        for j in (i + 1)..active_nodes.len() {
            let start: NodeId = active_nodes[i];
            let end: NodeId = active_nodes[j];
            if seen.contains(&start) && seen.contains(&end) {
                // O(1) * 2
                log::debug!(
                    "Skipping search for path [{}..{}] as it has already been checked.",
                    start,
                    end
                );
                continue;
            }
            let insertions = tree
                .bfs(start, end)
                .into_iter()
                .filter(|v| seen.insert(*v))
                .count();

            log::debug!(
                "Valid path found for [{}..{}], with {} steps",
                start,
                end,
                insertions
            );
        }
    });

    seen
}
