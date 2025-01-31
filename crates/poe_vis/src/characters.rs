use crate::{
    components::{NodeActive, NodeInactive},
    events::NodeActivationReq,
    resources::{ActiveCharacter, PathRepairRequired, RootNode},
};
use bevy::{color::Color, prelude::*, utils::HashMap};
use poe_tree::{character::Character, type_wrappings::NodeId};

pub struct CharacterPlugin;
impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_character);
        app.add_systems(PostStartup, activate_starting_nodes);

        log::debug!("CharacterPlugin plugin enabled");
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
