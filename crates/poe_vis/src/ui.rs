#![allow(dead_code, unused_variables, unused_imports)]
// Minimal example using bevy_egui instead of standard bevy UI:
use crate::{
    camera::CameraSettings,
    components::{NodeActive, NodeMarker},
    events::{MoveCameraReq, NodeDeactivationReq},
    resources::ActiveCharacter,
    PassiveTreeWrapper,
};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin}; // same components

pub struct UIPlugin; // our new EGUI-based plugin

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveNodeCounter>() // store node count
            .add_plugins(EguiPlugin)
            .add_systems(Update, update_active_nodecount) // track how many are active
            .add_systems(Update, egui_ui_system); // draw EGUI
    }
}

#[derive(Resource, Default)]
struct ActiveNodeCounter(pub usize);

// just count the active nodes
fn update_active_nodecount(
    active_nodes: Query<&NodeMarker, With<NodeActive>>,
    mut counter: Local<ActiveNodeCounter>,
) {
    counter.0 = active_nodes.iter().count();
}
fn egui_ui_system(
    counter: Res<ActiveNodeCounter>,
    tree: Res<PassiveTreeWrapper>,
    character: Res<ActiveCharacter>,
    active_nodes: Query<&NodeMarker, With<NodeActive>>,
    mut contexts: EguiContexts,
    mut deactivate_tx: EventWriter<NodeDeactivationReq>,
    settings: Res<CameraSettings>,
    mut move_camera_tx: EventWriter<MoveCameraReq>,
) {
    let ctx = contexts.ctx_mut();
    egui::SidePanel::right("rhs")
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("Current Path");
            ui.collapsing("Node details...", |ui| {
                ui.separator();
                ui.heading("Active Nodes");
                let root_id = character.starting_node;
                if let Some(root) = tree.nodes.get(&root_id) {
                    ui.horizontal(|ui| {
                        ui.label(&root.name);
                        if ui.small_button(format!("{}", root_id)).clicked() {
                            move_camera_tx.send(MoveCameraReq(Vec3::new(
                                root.wx,
                                -root.wy,
                                settings.min_zoom,
                            )));
                            log::trace!("Move2Node triggered...");
                        }
                    });
                }
                let actives: Vec<&NodeMarker> = active_nodes
                    .into_iter()
                    .filter(|nm| nm.0 != character.starting_node)
                    .collect();

                actives.iter().for_each(|nm| {
                    let poe_node = tree.nodes.get(&nm.0).unwrap();
                    ui.horizontal(|ui| {
                        ui.label(&poe_node.name);
                        if ui.small_button(format!("{}", nm.0)).clicked() {
                            move_camera_tx.send(MoveCameraReq(Vec3::new(
                                poe_node.wx,
                                -poe_node.wy,
                                settings.min_zoom,
                            )));
                            log::trace!("Move2Node triggered...");
                        }
                    });
                });
            });

            ui.separator();
            ui.heading(format!("{} Points Spent", active_nodes.iter().len()));
            ui.separator();

            if ui.button("Clear Active").clicked() {
                for nm in active_nodes.iter() {
                    if nm.0 != character.starting_node {
                        deactivate_tx.send(NodeDeactivationReq(nm.0));
                    }
                }
            }
        });
}

// fn egui_ui_system(
//     counter: Res<ActiveNodeCounter>,
//     tree: Res<PassiveTreeWrapper>,
//     character: Res<ActiveCharacter>,
//     active_nodes: Query<&NodeMarker, With<NodeActive>>,
//     mut contexts: EguiContexts,
//     mut deactivate_tx: EventWriter<NodeDeactivationReq>,
//     settings: Res<CameraSettings>,
//     mut move_camera_tx: EventWriter<MoveCameraReq>,
// ) {
//     let ctx = contexts.ctx_mut();

//     egui::SidePanel::right("rhs")
//         .resizable(true)
//         .show(ctx, |ui| {
//             ui.heading("Right Panel");
//             ui.collapsing("Something else", |ui| {
//                 ui.label("More details here.");
//             });

//             ui.heading("Active Nodes");
//             let actives: Vec<&NodeMarker> = active_nodes
//                 .into_iter()
//                 .filter(|nm| nm.0 != character.starting_node)
//                 .collect();

//             actives.iter().into_iter().for_each(|nm| {
//                 let poe_node = tree.nodes.get(&nm.0).unwrap(); // get data
//                 ui.horizontal(|ui| {
//                     ui.label(&poe_node.name);

//                     if ui.small_button(format!("{}", nm.0)).clicked() {
//                         move_camera_tx.send(MoveCameraReq(Vec3::new(
//                             poe_node.wx,       // node x
//                             -poe_node.wy,      // -1 * node wy
//                             settings.min_zoom, // we zoom them RIIIIIGHT in baby
//                         )));
//                         log::trace!("Move2Node triggered...");
//                     }
//                 });
//             });

//             ui.label(format!("Count: {}", actives.len()));
//             if ui.button("Clear Active").clicked() {
//                 for nm in actives {
//                     deactivate_tx.send(NodeDeactivationReq(nm.0));
//                 }
//             }
//         });
// }
