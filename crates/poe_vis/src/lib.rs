use bevy::prelude::*;

pub mod background_services;
pub mod camera;
pub mod components;
pub mod config;
pub mod controls;
pub mod nodes;

// Plugin to display nodes
pub struct PoeVis;
impl Plugin for PoeVis {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, crate::config::setup_character)
            .insert_resource(Time::<Fixed>::from_seconds(0.8))
            .add_systems(
                FixedUpdate,
                (background_services::pathfinding_system
                    .run_if(crate::background_services::node_active_changed)
                    .run_if(crate::background_services::sufficient_active_nodes)
                    .after(crate::controls::handle_node_clicks)),
            );

        app.insert_resource(nodes::NodeScaling {
            min_scale: 4.0,    // Nodes can shrink to 50% size
            max_scale: 8.0,    // Nodes can grow to 200% size
            base_radius: 60.0, // Should match your node radius
        })
        .add_plugins(crate::camera::PoeVisCameraPlugin)
        .add_systems(PreStartup, nodes::init_materials)
        .add_systems(
            Startup,
            (
                nodes::spawn_nodes,
                nodes::spawn_edges,
                nodes::adjust_node_sizes,
            ),
        )
        .add_systems(
            Update,
            (
                crate::controls::handle_node_clicks,
                crate::background_services::bg_edge_updater,
                nodes::update_materials,
            ),
        );

        app.add_systems(
            Update,
            (
                nodes::highlight_starting_node,
                nodes::update_materials.after(nodes::highlight_starting_node),
            ),
        );
    }
}
