use std::{collections::VecDeque, path::PathBuf};

use bevy::{
    ecs::event::EventId,
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_cosmic_edit::CosmicTextChanged;
use poe_tree::{character::Character, edges::Edge, type_wrappings::*};

use crate::{components::EdgeMarker, consts::SEARCH_THRESHOLD};

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
    pub fn is_moving(settings: Res<DragState>) -> bool {
        settings.active
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

    pub(crate) fn is_open(state: Res<SearchState>) -> bool {
        state.open
    }

    pub(crate) fn is_closed(state: Res<SearchState>) -> bool {
        !state.open
    }
}

#[derive(Resource, Default)]
pub struct VirtualPath {
    pub nodes: Vec<NodeId>,
    pub edges: Vec<EdgeMarker>,
}

impl VirtualPath {
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}
impl VirtualPath {
    pub fn contains_node(&self, node: NodeId) -> bool {
        // assuming self.nodes is sorted
        for &n in &self.nodes {
            if n > node {
                return false;
            } // gone past
            if n == node {
                return true;
            }
        }
        false
    }

    pub fn contains_edge(&self, edge: &EdgeMarker) -> bool {
        let (n1, n2) = match edge.0 < edge.1 {
            true => (edge.0, edge.1),
            false => (edge.1, edge.0),
        };

        for &EdgeMarker(s, e) in &self.edges {
            if s > n1 || e > n2 {
                return false;
            } // gone past
            if s == n1 && e == n2 {
                return true;
            }
        }
        false
    }

    /// NOTE: we keep the nodes sorted, so that when (in particular we look for contained edges), we're FAST.
    /// We do NOT sort the edges (there is no meaning to it in the context of the tree.)
    /*
    This change was observed, when we stopped sorting the edges.
    virtual path contains_edge (unsorted)
        time:   [22.916 ns 22.953 ns 22.993 ns]
        change: [-44.287% -43.952% -43.627%] (p = 0.00 < 0.05)
        Performance has improved.
     */
    pub fn sort(&mut self) {
        self.nodes.sort_unstable();
        // self.edges.sort_unstable_by_key(|em| {
        //     let (s, e) = em.as_tuple();
        //     (s, e)
        //
        // });
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct LastSaveLocation(pub PathBuf);

#[cfg(test)]
mod tests {
    use poe_tree::quick_tree;

    use crate::components::EdgeMarker;

    use super::VirtualPath;

    #[test]
    fn can_sort_virtual_path() {
        let tree = quick_tree();

        let mut vp = VirtualPath {
            nodes: tree.nodes.keys().cloned().collect(),
            edges: tree
                .get_edges()
                .into_iter()
                .map(|(start, end)| EdgeMarker(start, end))
                .collect(),
        };

        use rand::seq::SliceRandom;
        vp.nodes.shuffle(&mut rand::rng());
        vp.edges.shuffle(&mut rand::rng());

        vp.sort();

        assert!(vp.nodes.windows(2).all(|w| w[0] <= w[1]));

        assert!(vp.edges.windows(2).all(|w| {
            let (s1, e1) = w[0].as_tuple();
            let (s2, e2) = w[1].as_tuple();
            s1 < s2 || (s1 == s2 && e1 <= e2)
        }));
    }
}
