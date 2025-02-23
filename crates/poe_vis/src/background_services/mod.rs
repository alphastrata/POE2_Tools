mod edges;
mod generated;
mod misc;
mod nodes;
mod optimiser_utils;
mod paths;
mod virtual_paths;

use bevy::prelude::*;

use edges::*;
pub use generated::{parse_tailwind_color, tailwind_to_egui};
use misc::*;
use nodes::*;
use optimiser_utils::*;
pub use paths::clear;
use paths::*;

use crate::{
    events::*,
    resources::{
        ActiveCharacter, CameraSettings, JobStatus, Optimiser, PathRepairRequired, RootNode,
        Toggles,
    },
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
            (
                /* Only scan for edges when we KNOW the path is valid */
                scan_edges_for_active_updates.run_if(resource_equals(PathRepairRequired(false))),
                //deactivations:
                process_node_deactivations.run_if(on_event::<NodeDeactivationReq>),
                process_edge_deactivations,
                scan_edges_for_inactive_updates,
                /* happening all the time with camera moves. */
                adjust_node_sizes,
            )
                .after(clear), //
        );
        app.add_systems(
            Update,
            (
                //
                //activations:
                process_node_activations.run_if(on_event::<NodeActivationReq>),
                process_edge_activations,
                // Lock the rate we populate the virtual paths
                virtual_paths::populate_virtual_path
                    .run_if(on_event::<VirtualPathReq>.and(time_passed(0.080))),
                virtual_paths::process_virtual_paths.after(virtual_paths::populate_virtual_path),
                virtual_paths::clear_virtual_paths.run_if(
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
