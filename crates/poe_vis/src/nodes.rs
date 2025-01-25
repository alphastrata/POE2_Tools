use bevy::app::Plugin;

pub struct NodeInteractionPlugin;

impl Plugin for NodeInteractionPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        log::debug!("NodeInteraction plugin enabled");
    }
}

fn activate_starting_node() {
    todo!()
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

