use std::collections::VecDeque;

use bevy::{
    ecs::event::EventId,
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_cosmic_edit::CosmicTextChanged;
use poe_tree::{character::Character, type_wrappings::*};

use crate::consts::SEARCH_THRESHOLD;

#[derive(Resource)]
pub struct NodeScaling {
    pub min_scale: f32,
    pub max_scale: f32,
    pub base_radius: f32,
    pub hover_multiplier: f32,
    pub hover_fade_time: f32,
}

#[derive(Resource, Deref, DerefMut)]
pub struct ActiveCharacter {
    pub character: poe_tree::character::Character,
}

#[derive(Resource, DerefMut, Deref)]
pub struct RootNode(pub Option<NodeId>);

#[derive(Debug, serde::Deserialize, Default, Resource)]
pub struct UserConfig {
    pub colors: HashMap<String, String>,
    pub controls: HashMap<String, Vec<String>>,

    #[serde(skip_deserializing)]
    #[serde(default)]
    pub character: Character,
}

#[derive(Deref, DerefMut, Default, Resource, PartialEq, Eq, PartialOrd, Ord)]
pub struct PathRepairRequired(pub bool);

impl PathRepairRequired {
    pub(crate) fn request_path_repair(&mut self) {
        log::debug!("Path Repair requested");
        **self = true;
    }
    pub(crate) fn set_unrequired(&mut self) {
        log::debug!("Path Repair marked as unrequired");

        **self = false;
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct MouseSelecetedNodeHistory(pub VecDeque<NodeId>);

#[derive(Resource, Debug)]
pub struct SearchState {
    pub search_query: String,
    pub open: bool,
}

impl SearchState {
    pub(crate) fn should_search(state: Res<SearchState>) -> bool {
        state.search_query.len() >= SEARCH_THRESHOLD
    }
}
