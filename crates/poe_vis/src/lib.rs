//$ crates/poe_vis/src/lib.rs
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    sync::atomic::AtomicBool,
};
pub mod background_services;
pub mod camera;
pub(crate) mod debug;
pub mod drawing;
pub mod io;

use poe_tree::{character::Character, config::UserConfig, type_wrappings::NodeId, PassiveTree};

impl<'p> TreeVis<'p> {
    pub(crate) const BASE_RADIUS: f32 = 8.0;
    pub(crate) const NOTABLE_MULTIPLIER: f32 = 1.5; // Scale notable nodes
    pub(crate) const NAMELESS_MULTIPLIER: f32 = 1.0; // Scale nameless nodes
    pub(crate) const CAMERA_OFFSET: (f32, f32) = (-2_600.0, -1_300.0);

    pub fn new(
        passive_tree: &'p mut PassiveTree,
        user_config: UserConfig,
        current_character: Option<Character>,
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
}

pub struct TreeVis<'p> {
    camera: RefCell<(f32, f32)>,
    zoom: f32,
    passive_tree: &'p mut PassiveTree,
    hovered_node: Option<u32>,

    #[allow(unused)]
    // Fuzzy-search-related
    fuzzy_search_open: AtomicBool,
    search_query: String,
    search_results: Vec<NodeId>,

    // Path-finder-related
    start_node_id: NodeId,
    target_node_id: NodeId,

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

impl eframe::App for TreeVis<'_> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // bg updates:
        if self.requires_activation_check {
            log::debug!("Checking for active nodes & edges to highlight..");
            self.check_and_activate_edges();
            self.check_and_activate_nodes();
            self.requires_activation_check = false;
        }
        self.update_character();

        // IO
        self.handle_mouse(ctx);
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
                    self.repair_broken_paths();
                }
            }

            if input.key_pressed(egui::Key::Escape) {
                std::process::exit(0);
            }
        });

        self.draw_top_bar(ctx);
        self.draw_rhs_menu(ctx);

        //DEBUG:
        self.draw_debug_bar(ctx);

        // drawing
        self.redraw_tree(ctx);

        self.draw_color_and_highlights(ctx);

        //todo: draw top menu (open tree, char etc..)
    }
}

impl TreeVis<'_> {
    pub fn show_class_popup(&mut self, ctx: &egui::Context) {
        // Ensure character exists, or initialize with default
        let character = self
            .current_character
            .get_or_insert_with(Character::default);

        // let selected_class = &mut character.character_class;

        egui::Window::new("Choose Your Class")
            .collapsible(false)
            .resizable(false)
            .title_bar(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("Please select your class:");

                //TODO: match the tuples of opportunity to the right class.
                // // Dropdown for class selection
                // egui::ComboBox::from_label("Class")
                //     .selected_text(format!("{:?}", selected_class))
                //     .show_ui(ui, |ui| {
                //       LEVEL_ONE_NODES
                //         .into_iter()
                //         .for_each(|class| {
                //             ui.selectable_value(selected_class, class, format!("{:?}", class));
                //         });
                //     });
            });
    }
}
