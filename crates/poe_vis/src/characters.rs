use crate::{
    background_services::{active_nodes_changed, clear},
    components::{NodeActive, NodeInactive, NodeMarker},
    events::{ClearAll, LoadCharacterReq, NodeActivationReq, VirtualPathReq},
    resources::{ActiveCharacter, PathRepairRequired, RootNode},
};
use bevy::{color::Color, prelude::*, utils::HashMap};
use poe_tree::{character::Character, type_wrappings::NodeId};

pub struct CharacterPlugin;
impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_character);

        app.add_systems(PostStartup, activate_starting_nodes);

        app.add_systems(
            Update,
            activate_starting_nodes.run_if(on_event::<LoadCharacterReq>),
        );

        app.add_systems(PostUpdate, update_active_character.after(clear));

        log::debug!("CharacterPlugin plugin enabled");
    }
}
fn activate_starting_nodes(
    mut node_activator: EventWriter<NodeActivationReq>,
    mut path_repair: ResMut<PathRepairRequired>,
    active_character: ResMut<ActiveCharacter>,
    mut starting_node: ResMut<RootNode>,
) {
    starting_node.0 = Some(active_character.starting_node);

    node_activator.send(NodeActivationReq(active_character.starting_node));

    active_character.activated_node_ids.iter().for_each(|nid| {
        node_activator.send(NodeActivationReq(*nid));
    });
    path_repair.request_path_repair();
}

fn update_active_character(
    mut active_character: ResMut<ActiveCharacter>,
    actuals: Query<&NodeMarker, With<NodeActive>>,
) {
    active_character.activated_node_ids = actuals.into_iter().map(|nm| nm.0).collect();
    log::debug!("activated node updates pushed to Character");
}

fn setup_character(
    mut commands: Commands,
    all_node_entities: Query<(Entity, &crate::components::NodeMarker)>,
) {
    let character = Character::load_from_toml("data/character.toml").unwrap_or_default();

    log::debug!("Loaded Character {}", character.name,);

    // Set root node from character data
    commands.insert_resource(RootNode(Some(character.starting_node)));

    // Activate pre-defined nodes
    let node_entity_map: HashMap<NodeId, Entity> =
        all_node_entities.iter().map(|(e, m)| (m.0, e)).collect();

    for node_id in character.activated_node_ids.iter() {
        if let Some(&entity) = node_entity_map.get(node_id) {
            commands
                .entity(entity)
                .remove::<NodeInactive>()
                .insert(NodeActive);

            log::debug!("Node: {} activated.", node_id);
        }
    }

    // Store character as resource
    commands.insert_resource(ActiveCharacter { character });
}
