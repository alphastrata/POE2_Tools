use std::path;

use bevy::prelude::*;

use crate::{
    events::NodeActivationReq,
    resources::{ActiveCharacter, PathRepairRequired},
};

pub struct NodeInteractionPlugin;

impl Plugin for NodeInteractionPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Startup, activate_starting_nodes);
        log::debug!("NodeInteraction plugin enabled");
    }
}

fn activate_starting_nodes(
    mut node_activator: EventWriter<NodeActivationReq>,
    mut path_repair: ResMut<PathRepairRequired>,
    character: Res<ActiveCharacter>,
) {
    node_activator.send(NodeActivationReq(character.starting_node));

    character.activated_node_ids.iter().for_each(|nid| {
        node_activator.send(NodeActivationReq(*nid));
    });
    path_repair.request_path_repair();
}

fn scale_nodes() {
    todo!()
}

fn activate_nodes() {
    todo!()
}

fn deactivate_nodes() {
    todo!()
}

fn handle_node_select() {
    todo!()
}

fn handle_searched_node() {
    todo!()
}

fn handle_node_hover() {}
