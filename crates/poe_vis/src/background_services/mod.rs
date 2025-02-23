use bevy::prelude::*;

use crate::{
    events::*,
    resources::{
        ActiveCharacter, CameraSettings, JobStatus, Optimiser, PathRepairRequired, RootNode,
        Toggles,
    },
};

// Re-export necessary modules
mod edges;
mod generated;
mod misc;
mod nodes;
mod optimiser_utils;
mod paths;
mod virtual_paths;

pub use generated::{parse_tailwind_color, tailwind_to_egui};
pub use paths::clear;

// Main BGServicesPlugin, this is our wrapper plugin
pub struct BGServicesPlugin;

impl Plugin for BGServicesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            EssentialsPlugin,
            NodePlugin,
            EdgePlugin,
            PathingPlugin,
            OptimiserPlugin,
        ));

        log::debug!("BGServices plugin enabled");
    }
}

// Just a convenience 'holder' plugin for all the events and resources we need.
pub struct EssentialsPlugin;

impl Plugin for EssentialsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ClearAll>()
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
            .add_event::<NodeScaleReq>()
            .add_event::<OptimiseReq>()
            .add_event::<OverrideCharacterNodesReq>()
            .add_event::<SaveCharacterAsReq>()
            .add_event::<SaveCharacterReq>()
            .add_event::<ShowSearch>()
            .add_event::<SyncCharacterReq>()
            .add_event::<ThrowWarning>()
            .add_event::<VirtualPathReq>()
            .init_resource::<Toggles>()
            .insert_resource(Optimiser {
                results: Vec::new(),
                status: JobStatus::Available,
            })
            .insert_resource(PathRepairRequired(false));

        log::debug!("Core plugin enabled");
    }
}

// Plugin for Node Management
pub struct NodePlugin;

impl Plugin for NodePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                nodes::process_node_deactivations.run_if(on_event::<NodeDeactivationReq>),
                misc::adjust_node_sizes,
            )
                .after(clear),
        );
        app.add_systems(
            Update,
            (
                nodes::process_node_activations.run_if(on_event::<NodeActivationReq>),
                nodes::process_node_colour_changes.run_if(on_event::<NodeColourReq>),
                nodes::process_manual_node_highlights
                    .run_if(on_event::<ManualNodeHighlightWithColour>),
            ),
        );
        log::debug!("Node plugin enabled");
    }
}

// Plugin for Edge Management
pub struct EdgePlugin;

impl Plugin for EdgePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                edges::scan_edges_for_active_updates
                    .run_if(resource_equals(PathRepairRequired(false))),
                edges::process_edge_deactivations,
                edges::scan_edges_for_inactive_updates,
            )
                .after(clear),
        );
        app.add_systems(
            Update,
            (
                edges::process_edge_activations,
                edges::process_edge_colour_changes.run_if(on_event::<EdgeColourReq>),
                edges::process_manual_edge_highlights
                    .run_if(on_event::<ManualEdgeHighlightWithColour>),
            ),
        );
        log::debug!("Edge plugin enabled");
    }
}

// Plugin for Path and Virtual Path Logic
pub struct PathingPlugin;

impl Plugin for PathingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                virtual_paths::populate_virtual_path
                    .run_if(on_event::<VirtualPathReq>.and(misc::time_passed(0.080))),
                virtual_paths::process_virtual_paths.after(virtual_paths::populate_virtual_path),
                virtual_paths::clear_virtual_paths.run_if(
                    on_event::<ClearVirtualPath>
                        .or(on_event::<ClearAll>.or(CameraSettings::is_moving)),
                ),
                paths::path_repair
                    .run_if(resource_exists::<RootNode>)
                    .run_if(nodes::sufficient_active_nodes)
                    .run_if(
                        resource_equals(PathRepairRequired(true))
                            .or(resource_changed::<ActiveCharacter>),
                    ),
            ),
        );
        app.add_systems(Update, clear.run_if(on_event::<ClearAll>));
        log::debug!("Pathing plugin enabled");
    }
}

// Plugin for Optimiser Logic
pub struct OptimiserPlugin;

impl Plugin for OptimiserPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            optimiser_utils::populate_optimiser.run_if(on_event::<OptimiseReq>),
        );
        log::debug!("Optimiser plugin enabled");
    }
}
