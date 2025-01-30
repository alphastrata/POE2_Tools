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

#[derive(Resource)]
pub struct CameraSettings {
    pub drag_sensitivity: f32,
    pub zoom_sensitivity: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
    pub egui_has_lock: bool,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            drag_sensitivity: 10.0,
            zoom_sensitivity: 0.15,
            min_zoom: 3.10,
            max_zoom: 80.0,
            egui_has_lock: false,
        }
    }
}
impl CameraSettings {
    pub fn egui_has_lock(settings: Res<CameraSettings>) -> bool {
        settings.egui_has_lock
    }
    pub fn should_zoom(settings: Res<CameraSettings>) -> bool {
        !settings.egui_has_lock
    }
}

// Camera drag state
#[derive(Resource, Default)]
pub struct DragState {
    pub active: bool,
    pub start_position: Vec2,
}

#[derive(Resource, Deref, DerefMut)]
pub struct ActiveCharacter {
    pub character: poe_tree::character::Character,
}

#[derive(Resource, DerefMut, Deref)]
pub struct RootNode(pub Option<NodeId>);
impl RootNode {
    pub fn is_set(root: Res<RootNode>) -> bool {
        root.0.is_some()
    }
}

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

    /// Don't capture hotkey shortcuts etc when the UI for search is open.
    pub(crate) fn lock_shortcuts(state: Res<SearchState>) -> bool {
        !state.open
    }
}
