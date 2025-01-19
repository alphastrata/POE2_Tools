use std::{
    cell::RefCell,
    collections::{HashMap, HashSet}, sync::atomic::AtomicBool,
};
pub mod camera;
pub mod drawing;
pub(crate) mod debug;
pub mod background_services;
pub mod io;

use poo_tree::{config::{UserCharacter, UserConfig}, PassiveTree};

impl TreeVis<'_> {
    pub(crate) const BASE_RADIUS: f32 = 8.0;
    pub(crate) const NOTABLE_MULTIPLIER: f32 = 1.5; // Scale notable nodes
    pub(crate) const NAMELESS_MULTIPLIER: f32 = 1.0; // Scale nameless nodes
    pub(crate) const CAMERA_OFFSET: (f32, f32) = (-2_600.0, -1_300.0);
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
