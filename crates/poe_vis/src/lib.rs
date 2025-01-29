#![allow(dead_code, unused_imports)]
#[allow(clippy::type_complexity)]
use bevy::prelude::*;

use background_services::BGServicesPlugin;
use camera::PoeVisCameraPlugin;
use characters::CharacterPlugin;
use config::UserConfigPlugin;
use hotkeys::HotkeysPlugin;
use init_tree::TreeCanvasPlugin;
use materials::PoeVisMaterials;
use mouse::MouseControlsPlugin;
use nodes::NodeInteractionPlugin;
use overlays_n_popups::OverlaysAndPopupsPlugin;
use remote::RPCPlugin;
use search::SearchToolsPlugin;
use ui::UIPlugin;

//  mod shaders;
mod background_services;
mod camera;
mod characters;
mod components;
mod config;
mod consts;
mod edges;
mod events;
mod hotkeys;
mod init_tree;
mod materials;
mod mouse;
mod nodes;
mod overlays_n_popups;
mod remote;
mod resources;
mod search;

pub struct PoeVis;

impl Plugin for PoeVis {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((
            //TODO: CFG FLAG
            RPCPlugin,
            // ALWAYS
            BGServicesPlugin,
            PoeVisCameraPlugin,
            TreeCanvasPlugin,
            CharacterPlugin,
            PoeVisMaterials,
            MouseControlsPlugin,
            UserConfigPlugin,
            SearchToolsPlugin,
            HotkeysPlugin,
            OverlaysAndPopupsPlugin,
            NodeInteractionPlugin,
            // ShadersPlugin
            UIPlugin,
        ));
    }
}

#[derive(Resource)]
struct PassiveTreeWrapper {
    tree: poe_tree::PassiveTree,
}
impl std::ops::Deref for PassiveTreeWrapper {
    type Target = poe_tree::PassiveTree;

    fn deref(&self) -> &Self::Target {
        &self.tree
    }
}
impl std::ops::DerefMut for PassiveTreeWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tree
    }
}

pub mod ui {
    #![allow(dead_code, unused_variables, unused_imports)]
    // Minimal example using bevy_egui instead of standard bevy UI:
    use crate::{
        components::{NodeActive, NodeMarker},
        events::NodeDeactivationReq,
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

    // show a small EGUI panel
    fn egui_ui_system(
        counter: Res<ActiveNodeCounter>,
        active_nodes: Query<&NodeMarker, With<NodeActive>>,
        mut contexts: EguiContexts,
        mut deactivate_tx: EventWriter<NodeDeactivationReq>,
    ) {
        let ctx = contexts.ctx_mut();

        // top menu bar
        egui::TopBottomPanel::top("top_menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        std::process::exit(0);
                    }
                });
                ui.menu_button("Edit", |_| {});
                ui.menu_button("View", |_| {});
                ui.menu_button("Help", |_| {});
            });
        });

        // // collapsible left panel
        // egui::SidePanel::left("lhs")
        //     .resizable(true)
        //     .show(ctx, |ui| {
        //         ui.heading("Left Panel");
        //         ui.collapsing("Something", |ui| {
        //             ui.label("Details here.");
        //         });
        //     });

        // collapsible right panel
        egui::SidePanel::right("rhs")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Right Panel");
                ui.collapsing("Something else", |ui| {
                    ui.label("More details here.");
                });

                ui.heading("Active Nodes");
                ui.label(format!("Count: {}", active_nodes.iter().count()));
                if ui.button("Clear Active").clicked() {
                    active_nodes.into_iter().for_each(|nm| {
                        // commands.entity(ent).remove::<NodeActive>();
                        deactivate_tx.send(NodeDeactivationReq(**nm));
                    });
                }
            });
    }
}
