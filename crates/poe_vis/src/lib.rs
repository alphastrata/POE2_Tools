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
        app.add_plugins((
            crate::camera::PoeVisCameraPlugin,
            crate::controls::KeyboardControlsPlugin,
        ))
        .add_systems(PreStartup, crate::config::setup_character)
        .insert_resource(Time::<Fixed>::from_seconds(0.024)) // limit the pathfinding!
        .insert_resource(nodes::NodeScaling {
            min_scale: 4.0,         // Nodes can shrink to 50% size
            max_scale: 8.0,         // Nodes can grow to 200% size
            base_radius: 60.0,      // Should match your node radius
            hover_multiplier: 1.06, // Nodes that are hovered are increased by %3 of their size
            hover_fade_time: 0.120,
        });

        app.add_systems(
            FixedUpdate,
            background_services::pathfinding_system
                .run_if(crate::background_services::node_active_changed)
                .run_if(crate::background_services::sufficient_active_nodes)
                .after(crate::controls::handle_node_clicks),
        )
        .add_systems(PreStartup, nodes::init_materials)
        .add_systems(Startup, (nodes::spawn_nodes, nodes::spawn_edges))
        .add_systems(
            Update,
            (
                // nodes::adjust_node_sizes,
                crate::controls::handle_node_clicks,
                crate::background_services::bg_edge_updater,
                nodes::update_materials,
                nodes::highlight_starting_node,
            ),
        );

        // Update system ordering in your app builder
        app.add_systems(
            Update,
            (
                crate::nodes::hover::show_node_info,
                //
                nodes::hover::hover_started,
                nodes::hover::hover_ended,
                //
                crate::nodes::hover::handle_highlighted_active_nodes,
            )
                .chain(),
        );

        app.add_systems(Startup, nodes::hover::spawn_hover_text);
    }
}
