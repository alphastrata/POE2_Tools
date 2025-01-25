use bevy::prelude::{Plugin, Resource};

use background_services::BGServicesPlugin;
use camera::PoeVisCameraPlugin;
use crate::characters::CharacterPlugin;
use init_tree::TreeCanvasPlugin;

pub mod background_services;
pub mod camera;
pub mod characters;
pub mod components;
pub mod config;
pub mod events;
pub mod hotkeys;
pub mod materials;
pub mod resources;
// pub mod shaders;
pub mod init_tree;
pub mod overlays_n_popups;

pub struct PoeVis;

impl Plugin for PoeVis {
    fn build(&self, app: &mut bevy::prelude::App) {
        // app.insert_resource(UserConfig)
        // app.insert_resource(Character)

        app.add_plugins((
            BGServicesPlugin,
            PoeVisCameraPlugin,
            TreeCanvasPlugin,
            CharacterPlugin
            //  NodeInteractionPlugin,

            //  HotkeysPlugin,
            // ShadersPlugin
        ));
    }
}

#[derive(Resource)]
pub struct PassiveTreeWrapper {
    pub tree: poe_tree::PassiveTree,
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
