use std::path;

use bevy::prelude::*;

use crate::{
    events::NodeActivationReq,
    resources::{ActiveCharacter, PathRepairRequired},
};

pub struct NodeInteractionPlugin;

impl Plugin for NodeInteractionPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        log::debug!("NodeInteraction plugin enabled");
    }
}

fn handle_searched_node() {
    todo!()
}
