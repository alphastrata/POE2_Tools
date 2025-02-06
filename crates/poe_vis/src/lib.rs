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
pub mod components; // Pub because used in benchmarks
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
pub mod resources; // Pub because used in benchmarks
mod search;
mod ui;

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

// // top menu bar
// egui::TopBottomPanel::top("top_menu").show(ctx, |ui| {
//     egui::menu::bar(ui, |ui| {
//         ui.menu_button("File", |ui| {
//             if ui.button("Quit").clicked() {
//                 std::process::exit(0);
//             }
//         });
//         ui.menu_button("Edit", |_| {});
//         ui.menu_button("View", |_| {});
//         ui.menu_button("Help", |_| {});
//     });
// });

// // collapsible left panel
// egui::SidePanel::left("lhs")
//     .resizable(true)
//     .show(ctx, |ui| {
//         ui.heading("Left Panel");
//         ui.collapsing("Something", |ui| {
//             ui.label("Details here.");
//         });
//     });
