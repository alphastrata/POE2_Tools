#![allow(dead_code, unused_imports)]
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

mod background_services;
mod camera;
mod characters;
mod components;
mod config;
mod consts;
mod events;
mod hotkeys;
mod materials;
mod mouse;
mod resources;
//  mod shaders;
mod init_tree;
mod overlays_n_popups;

mod edges;
mod nodes;

pub mod remote;

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
            HotkeysPlugin,
            OverlaysAndPopupsPlugin,
            NodeInteractionPlugin,
            // ShadersPlugin
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
