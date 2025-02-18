use crate::{
    background_services::{active_nodes_changed, clear},
    components::{NodeActive, NodeInactive, NodeMarker},
    consts::DEFAULT_SAVE_PATH,
    events::{ClearAll, LoadCharacterReq, NodeActivationReq, SyncCharacterReq, VirtualPathReq},
    resources::{ActiveCharacter, LastSaveLocation, PathRepairRequired, RootNode},
};
use bevy::{color::Color, prelude::*, utils::HashMap};
use poe_tree::{character::Character, consts::get_char_starts_node_map, type_wrappings::NodeId};

pub struct CharacterPlugin;
impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LastSaveLocation(DEFAULT_SAVE_PATH.into()));

        app.add_systems(Startup, setup_character);

        // app.add_systems(PostStartup, activate_starting_nodes);

        app.add_systems(
            PostUpdate,
            (
                set_starting_node_based_on_character_class.run_if(
                    resource_exists::<ActiveCharacter>
                        .or(resource_exists_and_equals(RootNode(None))),
                ),
                update_active_character.run_if(
                    on_event::<SyncCharacterReq>
                        .or(on_event::<LoadCharacterReq>.or(ActiveCharacter::has_been_updated)),
                ),
            ),
        );

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
    active_character: ResMut<ActiveCharacter>,
    mut starting_node: ResMut<RootNode>,
) {
    starting_node.0 = Some(active_character.starting_node);

    node_activator.send(NodeActivationReq(active_character.starting_node));

    active_character.activated_node_ids.iter().for_each(|nid| {
        node_activator.send(NodeActivationReq(*nid));
    });
}

/// aliasing the `[activate_starting_nodes]` nodes logic again, but wrapped to more accurately reflect usage
fn update_active_from_character(
    node_activator: EventWriter<NodeActivationReq>,
    active_character: ResMut<ActiveCharacter>,
    starting_node: ResMut<RootNode>,
) {
    activate_starting_nodes(node_activator, active_character, starting_node);
}

fn update_active_character(
    // sync: EventReader<SyncCharacterReq>,
    mut active_character: ResMut<ActiveCharacter>,
    actuals: Query<&NodeMarker, With<NodeActive>>,
) {
    let start = active_character.starting_node;
    active_character.activated_node_ids = actuals.into_iter().map(|nm| nm.0).collect();
    active_character.activated_node_ids.insert(start);
    log::debug!(
        "{} activated node updates pushed to Character",
        active_character.activated_node_ids.len() + 1
    );
}

fn set_starting_node_based_on_character_class(
    mut active_character: ResMut<ActiveCharacter>,
    mut starting_node: ResMut<RootNode>,
) {
    log::debug!("Setting start_node");
    let options = get_char_starts_node_map();
    let class = active_character.character_class.as_str();
    let new = *options
        .get(class)
        .expect("it should be impossible to request a class we do NOT have a starting node for");
    active_character.starting_node = new;
    starting_node.0 = Some(new);
    log::debug!("start_node={:#?}", starting_node.0);
}

fn setup_character(mut commands: Commands) {
    let mut character = Character::load_from_toml("data/character.toml").unwrap_or_default();
    log::debug!("Loaded Character {}", character.name,);

    let root = character.root_node();
    character.activated_node_ids.insert(root);
    commands.insert_resource(RootNode(Some(character.root_node())));

    commands.insert_resource(ActiveCharacter { character });
}
