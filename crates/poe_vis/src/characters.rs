use crate::{
    background_services::{active_nodes_changed, clear},
    components::{NodeActive, NodeInactive, NodeMarker},
    consts::DEFAULT_SAVE_PATH,
    events::{ClearAll, LoadCharacterReq, NodeActivationReq, VirtualPathReq},
    resources::{ActiveCharacter, LastSaveLocation, PathRepairRequired, RootNode},
};
use bevy::{color::Color, prelude::*, utils::HashMap};
use poe_tree::{character::Character, consts::get_char_starts_node_map, type_wrappings::NodeId};

pub struct CharacterPlugin;
impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LastSaveLocation(DEFAULT_SAVE_PATH.into()));

        app.add_systems(
            Startup,
            (
                setup_character,
                set_starting_node_based_on_character_class.run_if(
                    resource_exists::<ActiveCharacter>
                        .or(resource_exists_and_equals(RootNode(None))),
                ),
            ),
        );

        app.add_systems(PostStartup, activate_starting_nodes);

        app.add_systems(
            Update,
            activate_starting_nodes
                .run_if(on_event::<LoadCharacterReq>.or(ActiveCharacter::has_been_updated)),
        );

        // app.add_systems(PostUpdate, update_active_character.after(clear));

        log::debug!("CharacterPlugin plugin enabled");
    }
}

impl ActiveCharacter {
    pub fn has_been_updated(
        active_char: Res<ActiveCharacter>,
        actual_active_nodes: Query<&NodeMarker, With<NodeActive>>,
    ) -> bool {
        log::debug!("Checking for updates to the ActiveCharacter");
        actual_active_nodes
            .into_iter()
            .all(|nid| active_char.activated_node_ids.contains(nid))
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

/// aliasing the `[activate_starting_nodes]` nodes logic again, but wrapped to more accurately reflect usage
fn update_active_from_character(
    node_activator: EventWriter<NodeActivationReq>,
    path_repair: ResMut<PathRepairRequired>,
    active_character: ResMut<ActiveCharacter>,
    starting_node: ResMut<RootNode>,
) {
    activate_starting_nodes(node_activator, path_repair, active_character, starting_node);
}

fn update_active_character(
    mut active_character: ResMut<ActiveCharacter>,
    actuals: Query<&NodeMarker, With<NodeActive>>,
) {
    active_character.activated_node_ids = actuals.into_iter().map(|nm| nm.0).collect();
    log::debug!("activated node updates pushed to Character");
}

fn set_starting_node_based_on_character_class(
    mut active_character: ResMut<ActiveCharacter>,
    mut starting_node: ResMut<RootNode>,
) {
    let options = get_char_starts_node_map();
    let class = active_character.character_class.as_str();
    let new = *options
        .get(class)
        .expect("it should be impossible to request a class we do NOT have a starting node for");
    active_character.starting_node = new;
    starting_node.0 = Some(new);
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
