use crate::{
    components::{NodeActive, NodeInactive},
    resources::{ActiveCharacter, RootNode},
};
use bevy::{color::Color, prelude::*, utils::HashMap};
use poe_tree::{character::Character, type_wrappings::NodeId};

pub struct CharacterPlugin;
impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_character);

        log::debug!("CharacterPlugin plugin enabled");
    }
}

pub fn setup_character(
    mut commands: Commands,
    all_node_entities: Query<(Entity, &crate::components::NodeMarker)>,
) {
    let character =
        Character::load_from_toml("data/character.toml").expect("Failed to load character data");

    log::debug!(
        "Character {} loaded with active nodes {:#?}",
        character.name,
        character.activated_node_ids
    );

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
        }
    }

    // Store character as resource
    commands.insert_resource(ActiveCharacter { character });
}
