#![allow(dead_code, unused_variables, unused_imports)]
// Minimal example using bevy_egui instead of standard bevy UI:
use crate::{
    components::{NodeActive, NodeMarker},
    events::{ClearAll, MoveCameraReq, NodeDeactivationReq, SaveCharacterReq},
    resources::{ActiveCharacter, CameraSettings},
    PassiveTreeWrapper,
};
use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align, SidePanel},
    EguiContexts, EguiPlugin,
};

use poe_tree::{nodes::PoeNode, PassiveTree};

pub struct UIPlugin; // our new EGUI-based plugin

#[derive(Resource, Default)]
struct UICapturesInput(pub bool);

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
            // space
            .init_resource::<UICapturesInput>()
            .init_resource::<ActiveNodeCounter>() // store node count
            // space
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
    active_nodes: Query<&NodeMarker, With<NodeActive>>,
    mut deactivate_tx: EventWriter<NodeDeactivationReq>,
    mut clear_all_tx: EventWriter<ClearAll>,
    mut move_camera_tx: EventWriter<MoveCameraReq>,
    mut save_tx: EventWriter<SaveCharacterReq>,
    counter: Res<ActiveNodeCounter>,
    tree: Res<PassiveTreeWrapper>,
    character: Res<ActiveCharacter>,
    mut contexts: EguiContexts,
    settings: Res<CameraSettings>,
) {
    let ctx = contexts.ctx_mut();
    SidePanel::right("rhs").resizable(true).show(ctx, |ui| {
        ui.heading("Active Nodes");
        ui.separator();
        ui.collapsing("Node details...", |ui| {
            ui.set_min_height(300.0); // Ensure enough space
            egui::ScrollArea::vertical().show(ui, |ui| {
                let root_id = character.starting_node;
                if let Some(root) = tree.nodes.get(&root_id) {
                    let root_stats = root.as_passive_skill(&tree).stats();
                    ui.horizontal(|ui| {
                        fmt_for_ui(root, &tree, ui);
                        ui.with_layout(egui::Layout::right_to_left(Align::RIGHT), |ui| {
                            if ui
                                .small_button("🏠")
                                .on_hover_text(format!("{:?}", root_stats))
                                .clicked()
                            {
                                move_camera_tx.send(MoveCameraReq(Vec3::new(
                                    root.wx,
                                    -root.wy,
                                    settings.min_zoom,
                                )));
                                log::trace!("Move2Node triggered...");
                            }
                        });
                    });
                }
                let active_nodes: Vec<&NodeMarker> = active_nodes
                    .into_iter()
                    .filter(|nm| nm.0 != character.starting_node)
                    .collect();
                active_nodes.iter().for_each(|nm| {
                    let poe_node = tree.nodes.get(&nm.0).unwrap();
                    let stats = poe_node.as_passive_skill(&tree).stats();
                    ui.horizontal(|ui| {
                        fmt_for_ui(poe_node, &tree, ui);
                        ui.with_layout(egui::Layout::right_to_left(Align::RIGHT), |ui| {
                            if ui
                                .small_button(format!("{}", nm.0))
                                .on_hover_text(format!("{:?}", stats))
                                .clicked()
                            {
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
            });
        });

        ui.separator();
        ui.heading(format!("{} Points Spent", active_nodes.iter().len()));
        ui.separator();
        if ui.button("Clear").clicked() {
            clear_all_tx.send(ClearAll);
        }

        if ui.button("Save").clicked() {
            save_tx.send(SaveCharacterReq);
        }

        //TODO: save_as
    });
}

fn fmt_for_ui(poe_node: &PoeNode, tree: &PassiveTree, ui: &mut egui::Ui) {
    let as_passive = poe_node.as_passive_skill(tree);
    if as_passive.is_notable() {
        ui.label(&poe_node.name);
    } else {
        let name = as_passive.name();
        let color = if name.to_lowercase().contains("dexterity") {
            egui::Color32::GREEN
        } else if name.to_lowercase().contains("strength") {
            egui::Color32::RED
        } else if name.to_lowercase().contains("intelligence") {
            egui::Color32::BLUE
        } else {
            egui::Color32::WHITE
        };
        //TODO: If it was a passive attribute we need to offer three buttons + for each..
        // When the user clicks those buttons, we need to update stats...
        ui.colored_label(color, name);
    }
}
