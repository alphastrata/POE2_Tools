use std::sync::{Arc, Mutex};

use crate::{
    background_services,
    components::{NodeActive, NodeInactive, NodeMarker},
    consts::DEFAULT_SAVE_PATH,
    events::{
        ClearAllReqReq, LoadCharacterReq, NodeActivationReq, NodeDeactivationReq,
        OverrideCharacterClassReq, OverrideCharacterNodesReq, SaveCharacterAsReq, SaveCharacterReq,
        SyncCharacterReq, VirtualPathReq,
    },
    resources::{ActiveCharacter, LastSaveLocation, PathRepairRequired, RootNode},
};
use bevy::{color::Color, prelude::*, utils::HashMap};
use poe_tree::{character::Character, consts::get_char_starts_node_map, type_wrappings::NodeId};

pub struct CharacterPlugin;
impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LastSaveLocation(DEFAULT_SAVE_PATH.into()));

        app.add_systems(Startup, (setup_character,));
        app.add_systems(
            PostStartup,
            set_starting_node_based_on_character_class
                .run_if(resource_exists_and_equals(RootNode(None))),
        );

        app.add_systems(
            Update,
            (
                handle_class_change.run_if(on_event::<OverrideCharacterClassReq>),
                //
                process_load_character
                    .run_if(on_event::<LoadCharacterReq>)
                    .after(background_services::clear),
                process_save_character
                    .run_if(on_event::<SaveCharacterReq>.or(on_event::<SaveCharacterAsReq>)),
                /* Users need to see paths magically illuminate */
            ),
        );

        app.add_systems(
            PostUpdate,
            (
                (
                    set_starting_node_based_on_character_class
                        .run_if(on_event::<OverrideCharacterClassReq>),
                    world_to_active_character.run_if(background_services::active_nodes_changed),
                ),
                override_char.run_if(on_event::<OverrideCharacterNodesReq>),
            ),
        );

        log::debug!("CharacterPlugin plugin enabled");
    }
}

fn override_char(
    mut active_mut: ResMut<ActiveCharacter>,
    mut req: EventReader<OverrideCharacterNodesReq>,
    mut node_activator: EventWriter<NodeActivationReq>,
    mut node_deactivator: EventWriter<NodeDeactivationReq>,
) {
    let previously_active_nodes = active_mut.activated_node_ids.clone();

    req.read().for_each(|r| {
        active_mut.activated_node_ids = r.iter().copied().collect();

        previously_active_nodes
            .iter()
            .filter(|old_id| !active_mut.activated_node_ids.contains(old_id))
            .for_each(|&old_id| {
                node_deactivator.send(NodeDeactivationReq(old_id));
            });

        active_mut.activated_node_ids.iter().for_each(|&new_id| {
            node_activator.send(NodeActivationReq(new_id));
        });
    });
}

impl ActiveCharacter {
    pub fn has_been_updated(
        active_char: Res<ActiveCharacter>,
        actual_active_nodes: Query<&NodeMarker, With<NodeActive>>,
    ) -> bool {
        log::trace!("Checking for updates to the ActiveCharacter");
        actual_active_nodes.iter().count() == active_char.activated_node_ids.len()
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

fn world_to_active_character(
    mut active_character: ResMut<ActiveCharacter>,
    actuals: Query<&NodeMarker, With<NodeActive>>,
) {
    let start = active_character.starting_node;
    active_character.activated_node_ids = actuals.into_iter().map(|nm| nm.0).collect();
    active_character.activated_node_ids.insert(start);
    log::trace!(
        "{} activated nodes",
        active_character.activated_node_ids.len()
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
fn handle_class_change(
    mut character: ResMut<ActiveCharacter>,
    mut events: EventReader<OverrideCharacterClassReq>,
) {
    for event in events.read() {
        character.character_class = event.0;
    }
}

fn setup_character(mut commands: Commands) {
    let mut character = Character::load_from_toml("data/character.toml").unwrap_or_default();
    log::debug!("Loaded Character {}", character.name,);

    let root = character.root_node();
    character.activated_node_ids.insert(root);
    commands.insert_resource(RootNode(Some(character.root_node())));

    commands.insert_resource(ActiveCharacter { character });
}

fn process_save_character(
    save: EventReader<SaveCharacterReq>,
    mut save_as: EventReader<SaveCharacterAsReq>, // "save as" event with a PathBuf
    mut last_save_loc: ResMut<LastSaveLocation>,
    mut active_character: ResMut<ActiveCharacter>,
    active_nodes: Query<&NodeMarker, With<NodeActive>>,
) {
    active_character.activated_node_ids = active_nodes.iter().map(|nm| **nm).collect();
    active_character.level = active_character.activated_node_ids.len() as u8;

    let path = if let Some(evt) = save_as.read().last() {
        **last_save_loc = (**evt).clone();
        (**evt).clone()
    } else if !save.is_empty() {
        last_save_loc.0.clone()
    } else {
        // Default fallback
        std::path::PathBuf::from(DEFAULT_SAVE_PATH)
    };

    if let Err(e) = active_character.save_to_toml(&path) {
        log::error!("{e}");
    }
    log::debug!("Character Saved to {path:?}");
}

fn process_load_character(
    mut loader: EventReader<LoadCharacterReq>,
    mut active_character: ResMut<ActiveCharacter>,
    mut root_node: ResMut<RootNode>,
) {
    println!("Load Character requested");
    loader.read().for_each(|req| {
        let path = &req.0;
        match path.extension().and_then(|s| s.to_str()) {
            Some("toml") => {
                // Use ours.
                match poe_tree::character::Character::load_from_toml(path) {
                    Some(character) => {
                        active_character.character = character;
                        println!("Load Character from OUR format finalised");
                    }
                    None => eprintln!("Failed to load TOML from {}", path.display()),
                }
            }
            Some("xml") => {
                // Assume XML is in PoB export format.
                match std::fs::read_to_string(path) {
                    Ok(xml_str) => {
                        match quick_xml::de::from_str::<poe_tree::pob_utils::POBCharacter>(&xml_str)
                        {
                            Ok(pob_char) => {
                                active_character.character = pob_char.into();
                                println!("Load Character from POB's format finalised");
                            }
                            Err(e) => log::error!("XML parse error in {}: {:?}", path.display(), e),
                        }
                    }
                    Err(e) => eprintln!("Failed to read {}: {:?}", path.display(), e),
                }
            }
            //TODO: throw UI error (There's an event for it ThrowWarning)
            Some(ext) => {
                log::error!("Unsupported file extension: {ext}");
            }
            None => {
                log::error!("Could not determine file extension for {}", path.display());
            }
        }
    });

    poe_tree::consts::LEVEL_ONE_NODES
        .iter()
        .flat_map(|v| active_character.activated_node_ids.get(v))
        .for_each(|v| {
            log::debug!("Resetting the root node to: {v}");

            **root_node = Some(*v)
        });
}

fn active_character_to_world(
    active_character: Res<ActiveCharacter>,
    mut deactivator: EventWriter<NodeDeactivationReq>,
    mut activator: EventWriter<NodeActivationReq>,
    query: Query<(Entity, &NodeMarker)>,
) {
    log::trace!("Updating character.");

    let active = Arc::new(Mutex::new(&mut activator));
    let deactive = Arc::new(Mutex::new(&mut deactivator));
    query.par_iter().for_each(|(_ent, nm)| {
        // We'll just be sloppy and potentially send activation requests to nodes that _may_
        // already BE active, and so on.
        match active_character.activated_node_ids.contains(&nm.0) {
            true => {
                active.lock().unwrap().send(NodeActivationReq(nm.0));
            }
            false => {
                deactive.lock().unwrap().send(NodeDeactivationReq(nm.0));
            }
        }
    });
}
