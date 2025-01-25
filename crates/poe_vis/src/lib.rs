#![allow(dead_code, unused_imports)]
use bevy::prelude::*;

use hotkeys::HotkeysPlugin;
use crate::characters::CharacterPlugin;
use background_services::BGServicesPlugin;
use camera::PoeVisCameraPlugin;
use init_tree::TreeCanvasPlugin;
use materials::PoeVisMaterials;

mod background_services;
mod camera;
mod characters;
mod components;
mod config;
mod consts;
mod events;
mod hotkeys;
mod materials;
mod resources;
//  mod shaders;
mod init_tree;
mod overlays_n_popups;

mod nodes;
mod edges;

 pub struct PoeVis;

impl Plugin for PoeVis {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((
            BGServicesPlugin,
            PoeVisCameraPlugin,
            // TreeCanvasPlugin,
            CharacterPlugin,
            PoeVisMaterials,
            HotkeysPlugin,
            //  NodeInteractionPlugin,

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
