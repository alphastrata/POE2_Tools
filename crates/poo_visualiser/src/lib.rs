//$ crates/poo_visualiser/src/lib.rs
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    sync::atomic::{AtomicBool, AtomicUsize},
};

use poo_tree::{config::{UserCharacter, UserConfig}, PassiveTree};

impl TreeVis<'_> {
    pub(crate) const ACTIVE_NODE_COUNT: AtomicUsize = AtomicUsize::new(0);
    pub(crate) const BASE_RADIUS: f32 = 8.0;
    pub(crate) const NOTABLE_MULTIPLIER: f32 = 1.5; // Scale notable nodes
    pub(crate) const NAMELESS_MULTIPLIER: f32 = 1.0; // Scale nameless nodes
    pub(crate) const CAMERA_OFFSET: (f32, f32) = (-2_600.0, -1_300.0);
}
pub mod camera {
    use std::{
        cell::RefCell,
        collections::{HashMap, HashSet},
        sync::atomic::AtomicBool,
    };

    use poo_tree::{config::{UserCharacter, UserConfig}, PassiveTree};

    use super::*;
  
    // Helper Functions
    impl<'p> TreeVis<'p> {
        // Node size constants

        pub fn new(
            passive_tree: &'p mut PassiveTree,
            user_config: UserConfig,
            current_character: Option<UserCharacter>,
        ) -> Self {
            Self {
                camera: RefCell::new(Self::CAMERA_OFFSET),
                zoom: 0.09,
                passive_tree,
                hovered_node: None, // No node hovered initially

                // Fuzzy-search-related
                fuzzy_search_open: AtomicBool::new(false), // Search not open initially
                search_query: String::new(),               // Empty search query
                search_results: Vec::new(),                // No search results initially

                // Path-finder-related
                start_node_id: 0,             // Default to the root or initial node
                target_node_id: 0,            // Default to no target node
                highlighted_path: Vec::new(), // No path initially
                active_edges: HashSet::new(), // No edges highlighted initially
                active_nodes: HashSet::new(),

                current_character,
                last_save_time: std::time::Instant::now(), // Set to the current time

                user_config,
                controls: HashMap::new(),
                requires_activation_check: false,
            }
        }

        pub fn move_camera_to_node(&self, node_id: usize) {
            if let Some(node) = self.passive_tree.nodes.get(&node_id) {
                let mut camera = self.camera.borrow_mut();
                camera.0 = node.wx as f32;
                camera.1 = node.wy as f32;

                log::debug!(
                    "Camera centered on node ID: {} at world position: ({:.2}, {:.2})",
                    node_id,
                    node.wx,
                    node.wy
                );
            }
        }
        pub fn go_to_node(&self, id: usize) {
            self.move_camera_to_node(id);
            // self.disable_fuzzy_search();
        }

        pub fn save_character(&mut self) {
            if let Some(character) = &self.current_character {
                character.save_to_toml("data/last_character.toml");
                self.last_save_time = std::time::Instant::now();
            }
        }

        #[allow(unused)]
        pub fn auto_save_character(&mut self) {
            if let Some(character) = &self.current_character {
                if self.last_save_time.elapsed().as_secs() >= 5 {
                    self.save_character();
                }
            }
        }

        pub fn load_character(&mut self, path: &str) {
            self.current_character = UserCharacter::load_from_toml(path);
        }

        pub(crate) const ZOOM_MIN: f32 = 0.0; // Minimum zoom level
        pub(crate) const ZOOM_MAX: f32 = 1.0; // Maximum zoom level
        pub(crate) const ZOOM_STEP: f32 = 0.0001; // Step size for zoom changes
        pub fn current_zoom_level(&self) -> f32 {
            self.zoom
        }

        /// Translate the camera based on mouse drag input.
        pub fn translate_camera(&mut self, dx: f32, dy: f32) {
            let mut camera = self.camera.borrow_mut();
            camera.0 += dx / self.zoom; // Adjust for current zoom level
            camera.1 += dy / self.zoom;
        }

        /// Adjust the zoom level based on raw scroll input.
        pub fn adjust_zoom(&mut self, scroll: f32, mouse_pos: egui::Pos2) {
            let new_zoom =
                (self.zoom + scroll * Self::ZOOM_STEP).clamp(Self::ZOOM_MIN, Self::ZOOM_MAX);

            if (new_zoom - self.zoom).abs() > f32::EPSILON {
                // Calculate the scaling factor
                let _scale = new_zoom / self.zoom;

                // Adjust the camera to center zooming around the mouse position
                let screen_center_x = self.camera.borrow().0 + mouse_pos.x / self.zoom;
                let screen_center_y = self.camera.borrow().1 + mouse_pos.y / self.zoom;

                let new_camera_x = screen_center_x - mouse_pos.x / new_zoom;
                let new_camera_y = screen_center_y - mouse_pos.y / new_zoom;

                // Update camera and zoom
                *self.camera.borrow_mut() = (new_camera_x, new_camera_y);
                self.zoom = new_zoom;
            }
        }

        pub fn world_to_screen_x(&self, wx: f64) -> f32 {
            (wx as f32 - self.camera.borrow().0) * self.zoom + 500.0
        }

        pub fn world_to_screen_y(&self, wy: f64) -> f32 {
            (wy as f32 - self.camera.borrow().1) * self.zoom + 500.0
        }

        pub fn screen_to_world_x(&self, sx: f32) -> f64 {
            ((sx - 500.0) / self.zoom + self.camera.borrow().0) as f64
        }

        pub fn screen_to_world_y(&self, sy: f32) -> f64 {
            ((sy - 500.0) / self.zoom + self.camera.borrow().1) as f64
        }
    }
}

pub mod drawing;

pub(crate) mod debug;

pub mod background_services;

pub mod io;

pub(crate) struct NonInteractiveAreas {}
pub struct TreeVis<'p> {
    camera: RefCell<(f32, f32)>,
    zoom: f32,
    passive_tree: &'p mut PassiveTree,
    hovered_node: Option<usize>,

    // Fuzzy-search-related
    fuzzy_search_open: AtomicBool,
    search_query: String,
    search_results: Vec<usize>,

    // Path-finder-related
    start_node_id: usize,
    target_node_id: usize,
    highlighted_path: Vec<usize>,

    /// Store edges of the current path
    // NOTE: mostly used for drawing.
    active_edges: HashSet<(usize, usize)>,
    active_nodes: HashSet<usize>,

    // Config-driven colours
    current_character: Option<UserCharacter>,
    last_save_time: std::time::Instant,

    user_config: UserConfig,

    /// Mapped controls from self.user_config
    #[allow(unused)]
    controls: HashMap<String, egui::Key>,

    requires_activation_check: bool,
}

impl eframe::App for TreeVis<'_> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // IO
        self.handle_mouse(ctx);

        if self.requires_activation_check {
            self.check_and_activate_nodes();
            self.check_and_activate_edges();
            self.requires_activation_check = false;
        }

        //DEBUG:
        self.draw_debug_bar(ctx);

        // Example: Process node hovering
        if let Some(hovered_node_id) = self.get_hovered_node(ctx) {
            self.hover_node(hovered_node_id);
        }

        // Example: Check and activate nodes if target node changes
        if let Some(target_node_id) = self.get_target_node() {
            self.select_node(target_node_id);
        }

        ctx.input(|input| {
            if let Some(hovered) = self.hovered_node {
                if input.pointer.primary_clicked() {
                    self.click_node(hovered);
                }
            }

            if input.key_pressed(egui::Key::Escape) {
                std::process::exit(0);
            }
        });

        // drawing
        self.redraw_tree(ctx);
        // TODO: maybe we highlight in the redraw_tree() call?
        self.draw_color_and_highlights(ctx);

        //TODO: draw rhs menu
        self.draw_rhs_menu(ctx);

        //todo: draw top menu (open tree, char etc..)
    }
}
