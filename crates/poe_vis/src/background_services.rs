use bevy::prelude::*;

use crate::{components::*, events::*, resources::*};

pub struct BGServicesPlugin;

impl Plugin for BGServicesPlugin {
    fn build(&self, app: &mut App) {

        app.insert_resource(NodeScaling {
            min_scale: 4.0,         // Nodes can shrink to 50% size
            max_scale: 8.0,         // Nodes can grow to 200% size
            base_radius: 60.0,      // Should match your node radius
            hover_multiplier: 1.06, // Nodes that are hovered are increased by %3 of their size
            hover_fade_time: 0.120,
        });

        app.add_event::<ScaleNode>().add_event::<ColourNode>();

        app.add_systems(
            Update,
            //TODO: rate-limiting
            (
                process_scale_requests, 
                process_colour_change_requests, 
                adjust_node_sizes 
            ),
        );
    }
}

// Conditional Helpers:
fn node_active_changed(query: Query<(), Changed<NodeActive>>) -> bool {
    !query.is_empty()
}

fn edge_active_changed(query: Query<(), Changed<EdgeActive>>) -> bool {
    !query.is_empty()
}

fn sufficient_active_nodes(query: Query<&NodeMarker, With<NodeActive>>) -> bool {
    query.iter().count() >= 2 // Only run if at least 2 nodes are active
}


// BG SERVICES:
fn process_scale_requests(
    mut scale_events: EventReader<ScaleNode>,
    mut transforms: Query<&mut Transform>,
) {
    scale_events
        .read()
        .for_each(|ScaleNode(entity, new_scale)| {
            if let Ok(mut t) = transforms.get_mut(*entity) {
                t.scale = Vec3::splat(*new_scale);
            }
        });
}

fn process_colour_change_requests(
    mut colour_events: EventReader<ColourNode>,
    mut materials_q: Query<&mut MeshMaterial2d<ColorMaterial>>,
) {
    colour_events.read().for_each(|ColourNode(entity, mat)| {
        if let Ok(mut m) = materials_q.get_mut(*entity) {
            m.0 = mat.clone_weak();
        }
    });
}

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
