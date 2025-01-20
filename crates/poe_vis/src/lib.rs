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

use poe_tree::{
    character::{Character, CharacterClass},
    config::UserConfig,
    consts::CHAR_START_NODES,
    PassiveTree,
};

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
    current_character: Option<Character>,
    last_save_time: std::time::Instant,

    user_config: UserConfig,

    /// Mapped controls from self.user_config
    #[allow(unused)]
    controls: HashMap<String, egui::Key>,

    requires_activation_check: bool,
}

impl eframe::App for TreeVis<'_> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check if a character is loaded, and show class selection popup if not
        if self.current_character.is_none() || self.start_node_id == 0usize {
            self.show_class_popup(ctx);
        }

        // IO
        self.handle_mouse(ctx);

        if self.requires_activation_check {
            log::debug!("Checking for active nodes & edges to highlight..");
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

impl TreeVis<'_> {
    pub fn show_class_popup(&mut self, ctx: &egui::Context) {
        // Ensure character exists, or initialize with default
        let character = self
            .current_character
            .get_or_insert_with(Character::default);

        let selected_class = &mut character.character_class;

        egui::Window::new("Choose Your Class")
            .collapsible(false)
            .resizable(false)
            .title_bar(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("Please select your class:");

                // Dropdown for class selection
                egui::ComboBox::from_label("Class")
                    .selected_text(format!("{:?}", selected_class))
                    .show_ui(ui, |ui| {
                        [
                            CharacterClass::Monk,
                            CharacterClass::Sorceress,
                            CharacterClass::Witch,
                            CharacterClass::Warrior,
                            CharacterClass::Mercenary,
                            CharacterClass::Ranger,
                        ]
                        .into_iter()
                        .for_each(|class| {
                            ui.selectable_value(selected_class, class, format!("{:?}", class));
                        });
                    });

                // Update the character's class and compute adjacent nodes
                let class_starting_node = CHAR_START_NODES[(*selected_class) as usize];
                let starting_node_candidates =
                    self.passive_tree.find_adjacent_nodes(class_starting_node);
                dbg!(starting_node_candidates.len());

                let starting_node_candidates: Vec<usize> = self
                    .passive_tree
                    .all_nodes_with_distance(class_starting_node, 1)
                    .into_iter()
                    .flatten()
                    .collect();
                dbg!(starting_node_candidates.len());

                let mut selected_starting_node = starting_node_candidates.first().unwrap(); //NOTE: should never be non-zero

                egui::ComboBox::from_label("Starting Node")
                    .selected_text(format!("Node {:?}", selected_starting_node))
                    .show_ui(ui, |ui| {
                        (&starting_node_candidates).into_iter().for_each(|node| {
                            ui.selectable_value(
                                &mut selected_starting_node,
                                node,
                                format!("Node {}", node),
                            );
                        });
                    });

                log::debug!(
                    "Selected Class: {:?}, Selected Starting Node: {}",
                    selected_class,
                    selected_starting_node
                );

                // Confirm button
                if ui.button("Confirm").clicked() {
                    log::debug!(
                        "Confirmed Class: {:?}, Adjacent Nodes: {:?}",
                        selected_class,
                        starting_node_candidates
                    );
                }
            });
    }
}
