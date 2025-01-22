//!$ crates/poe_vis/src/lib.rs
use core::panic;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    sync::atomic::{AtomicBool, Ordering},
};
pub mod background_services;
pub mod camera;
pub mod config;
pub(crate) mod debug;
pub mod drawing;
pub mod io;

use config::UserConfig;
use poe_tree::{character::Character, type_wrappings::NodeId, PassiveTree};

impl<'p> TreeVis<'p> {
    pub fn new(
        passive_tree: &'p mut PassiveTree,
        user_config: UserConfig,
        current_character: Option<Character>,
    ) -> Self {
        Self {
            camera: RefCell::new(Self::CAMERA_OFFSET),
            zoom: Self::DEFAULT_STARTING_CAMERA_ZOOM.into(),
            passive_tree,

            // For virtual-paths && Hover
            hovered_node: None,           // No node hovered initially
            highlighted_path: Vec::new(), // No path initially

            // Fuzzy-search-related
            fuzzy_search_open: AtomicBool::new(false), // Search not open initially
            search_query: String::new(),               // Empty search query
            search_results: Vec::new(),                // No search results initially

            // Path-finder-related
            start_node_id: 0,             // Default to the root or initial node
            active_edges: HashSet::new(), // No edges highlighted initially
            active_nodes: HashSet::new(),

            //unused?
            selected_node_id: 0, // Default to no target node

            current_character,
            last_save_time: std::time::Instant::now(), // Set to the current time

            user_config,
            controls: HashMap::new(),

            // Rerun all the checks etc to look for, correct and so on broken paths.
            requires_activation_check: false,
        }
    }

    pub fn is_fuzzy_search_open(&self) -> bool {
        self.fuzzy_search_open.load(Ordering::Relaxed)
    }

    pub fn open_fuzzy_search(&self) {
        self.fuzzy_search_open.swap(true, Ordering::Acquire);
        // Let em see...
        self.set_zoom_level(0.035); //approximately the whole tree at 1080p
    }

    pub fn close_fuzzy_search(&self) {
        self.fuzzy_search_open.swap(false, Ordering::Acquire);
    }
}
impl TreeVis<'_> {
    pub fn initialize_camera(&mut self, ctx: &egui::Context) {
        // Get screen dimensions
        let screen_rect = ctx.screen_rect();
        let screen_width = screen_rect.width();
        let screen_height = screen_rect.height();

        // Set camera to center on (0, 0)
        *self.camera.borrow_mut() = (0.0, 0.0);

        // Calculate zoom level to fit the entire tree
        let tree_width = 12000.0; // Replace with actual tree width in world coordinates
        let tree_height = 12000.0; // Replace with actual tree height in world coordinates
        let zoom_x = screen_width / tree_width;
        let zoom_y = screen_height / tree_height;
        let optimal_zoom = zoom_x.min(zoom_y);
        dbg!(optimal_zoom);

        *self.zoom.borrow_mut() = optimal_zoom;
    }
}

pub struct TreeVis<'p> {
    camera: RefCell<(f32, f32)>,
    zoom: RefCell<f32>,
    passive_tree: &'p mut PassiveTree,
    hovered_node: Option<NodeId>,

    #[allow(unused)]
    // Fuzzy-search-related
    fuzzy_search_open: AtomicBool,
    search_query: String,
    search_results: Vec<NodeId>,

    // Path-finder-related
    start_node_id: NodeId,
    selected_node_id: NodeId,

    // It _is_ possible to have a node highlighted without levelling to it, with gear, which is why this is already separate from the active_nodes
    highlighted_path: Vec<NodeId>,

    /// Store edges of the current path
    // NOTE: mostly used for drawing.
    active_edges: HashSet<(NodeId, NodeId)>,
    active_nodes: HashSet<NodeId>,

    // Config-driven colours
    current_character: Option<Character>,

    #[allow(unused)]
    last_save_time: std::time::Instant,

    user_config: UserConfig,

    /// Mapped controls from self.user_config
    #[allow(unused)]
    controls: HashMap<String, egui::Key>,

    requires_activation_check: bool,
}

static CAMERA_INIT: AtomicBool = AtomicBool::new(false);

impl eframe::App for TreeVis<'_> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // do once:
        // it'd be nice if egui had a way to ... do this nicer.
        // if !CAMERA_INIT.load(Ordering::Acquire) {
        //     self.initialize_camera(ctx);
        //     CAMERA_INIT.swap(true, Ordering::SeqCst);
        // }

        // bg updates:
        if self.requires_activation_check {
            log::debug!("Checking for active nodes & edges to highlight..");

            self.repair_broken_paths();

            self.check_and_activate_nodes();

            self.check_and_activate_edges();

            self.requires_activation_check = false;

            // Is it true the number of nodes / 2 == the number of edges?
        }
        self.update_character();

        // IO, no mouse when searching...
        self.handle_keyboard(ctx);
        // IO, keyboard:
        self.handle_mouse(ctx);
        if let Some(hovered_node_id) = self.get_hovered_node(ctx) {
            self.hover_node(hovered_node_id);
        }

        // // Example: Check and activate nodes if target node changes
        // if let Some(target_node_id) = self.get_target_node() {
        //     self.select_node(target_node_id);
        // }

        if self.is_fuzzy_search_open() {
            self.show_fuzzy_search_popup(ctx);
        }

        ctx.input(|input| {
            if let Some(hovered) = self.hovered_node {
                if input.pointer.primary_clicked() {
                    self.click_node(hovered);
                    self.repair_broken_paths();
                }
            }
        });
        // DRAWING:
        self.draw_top_bar(ctx);
        // self.draw_rhs_menu(ctx);

        //DEBUG:
        self.draw_debug_bar(ctx);

        self.redraw_tree(ctx);

        //todo: draw top menu (open tree, char etc..)
    }
}

impl TreeVis<'_> {
    pub fn show_class_popup(&mut self, ctx: &egui::Context) {}
}
