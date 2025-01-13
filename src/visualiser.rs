//$ src\visualiser.rs
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    default::Default,
    hash::Hash,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::config::UserConfig;
use crate::{
    config::{parse_color, UserCharacter},
    data::PassiveTree,
};

impl eframe::App for TreeVis {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Main menu bar for File -> Open
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        // Open file dialog (blocking)
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            if let Some(character) =
                                UserCharacter::load_from_toml(path.to_str().unwrap())
                            {
                                self.current_character = Some(character);
                            }
                        }
                    }
                });
            });
        });

        // Main draw area
        egui::CentralPanel::default().show(ctx, |ui| {
            let available = ui.available_size();
            let (rect, _resp) = ui.allocate_at_least(available, egui::Sense::drag());
            let painter = ui.painter_at(rect);

            // Movement controls (disabled during fuzzy search)
            if !self.is_fuzzy_search_open() {
                let step = 20.0 / self.zoom;
                if let Some(&key) = self.controls.get("move_up") {
                    if ui.input(|i| i.key_down(key)) {
                        self.camera.borrow_mut().1 -= step;
                    }
                }
                if let Some(&key) = self.controls.get("move_down") {
                    if ui.input(|i| i.key_down(key)) {
                        self.camera.borrow_mut().1 += step;
                    }
                }
                if let Some(&key) = self.controls.get("move_left") {
                    if ui.input(|i| i.key_down(key)) {
                        self.camera.borrow_mut().0 -= step;
                    }
                }
                if let Some(&key) = self.controls.get("move_right") {
                    if ui.input(|i| i.key_down(key)) {
                        self.camera.borrow_mut().0 += step;
                    }
                }
            }

            // Mouse wheel zoom
            let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
            if scroll_delta != 0.0 {
                self.zoom += 0.001 * scroll_delta;
                self.zoom = self.zoom.clamp(0.01, 100.0);
            }

            // Highlight hovered nodes and toggle activation
            if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                if rect.contains(pos) {
                    let mx = self.screen_to_world_x(pos.x - rect.min.x);
                    let my = self.screen_to_world_y(pos.y - rect.min.y);
                    self.update_hover(mx, my);
                    if ui.input(|i| i.pointer.primary_clicked()) {
                        if let Some(id) = self.hovered_node {
                            if let Some(node) = self.passive_tree.nodes.get_mut(&id) {
                                node.active = !node.active;
                            }
                        }
                    }
                }
            }

            // Draw edges
            for (&nid, node) in &self.passive_tree.nodes {
                for &other_id in &node.connections {
                    let is_on_path = self.active_edges.contains(&(nid, other_id));
                    let stroke_color = if is_on_path {
                        parse_color(
                            self.user_config
                                .colors
                                .get("yellow")
                                .unwrap_or(&"#FFFF00".to_string()),
                        )
                    } else {
                        parse_color(
                            self.user_config
                                .colors
                                .get("default")
                                .unwrap_or(&"#3C3C3C".to_string()),
                        )
                    };
                    painter.line_segment(
                        [
                            egui::pos2(
                                self.world_to_screen_x(node.wx),
                                self.world_to_screen_y(node.wy),
                            ),
                            egui::pos2(
                                self.world_to_screen_x(self.passive_tree.nodes[&other_id].wx),
                                self.world_to_screen_y(self.passive_tree.nodes[&other_id].wy),
                            ),
                        ],
                        egui::Stroke::new(2.0, stroke_color),
                    );
                }
            }

            // Draw nodes
            let base_node_size = 6.0;
            for node in self.passive_tree.nodes.values() {
                let sx = self.world_to_screen_x(node.wx) + rect.min.x;
                let sy = self.world_to_screen_y(node.wy) + rect.min.y;
                let node_size = base_node_size * (1.0 + self.zoom * 0.1);

                let color = if node.active {
                    parse_color(
                        self.user_config
                            .colors
                            .get("green")
                            .unwrap_or(&"#29D398".to_string()),
                    )
                } else {
                    parse_color(
                        self.user_config
                            .colors
                            .get("all_nodes")
                            .unwrap_or(&"#3C3C3C".to_string()),
                    )
                };

                painter.circle_filled(egui::pos2(sx, sy), node_size, color);
            }

            // Highlight nodes from fuzzy search
            if self.is_fuzzy_search_open() {
                for &node_id in &self.search_results {
                    if let Some(node) = self.passive_tree.nodes.get(&node_id) {
                        let sx = self.world_to_screen_x(node.wx) + rect.min.x;
                        let sy = self.world_to_screen_y(node.wy) + rect.min.y;
                        painter.circle_stroke(
                            egui::pos2(sx, sy),
                            10.0,
                            egui::Stroke::new(
                                2.0,
                                parse_color(
                                    self.user_config
                                        .colors
                                        .get("purple")
                                        .unwrap_or(&"#EE64AC".to_string()),
                                ),
                            ),
                        );
                    }
                }
            }
        });

        // Fuzzy search UI
        if ctx.input(|i| i.key_pressed(egui::Key::F)) {
            self.enable_fuzzy_search();
        }
        if self.is_fuzzy_search_open() {
            egui::Window::new("Fuzzy Search")
                .collapsible(true)
                .show(ctx, |ui| {
                    let resp = ui.text_edit_singleline(&mut self.search_query);
                    if resp.changed() {
                        self.search_results =
                            self.passive_tree.fuzzy_search_nodes(&self.search_query);
                    }
                    egui::CollapsingHeader::new("Results").show(ui, |ui| {
                        for &id in &self.search_results {
                            let node = &self.passive_tree.nodes[&id];
                            if ui.selectable_label(false, &node.name).double_clicked() {
                                self.go_to_node(id);
                            }
                        }
                    });
                });
        }
    }
}

pub struct TreeVis {
    camera: RefCell<(f32, f32)>,
    zoom: f32,
    passive_tree: crate::data::PassiveTree,
    hovered_node: Option<usize>,

    // Fuzzy-search-related
    fuzzy_search_open: AtomicBool,
    search_query: String,
    search_results: Vec<usize>,

    // Path-finder-related
    start_node_id: usize,
    target_node_id: usize,
    path: Vec<usize>,

    /// Store edges of the current path
    active_edges: HashSet<(usize, usize)>,

    // Config-driven colours
    color_map: HashMap<String, String>,

    // for multi-step pathing
    path_nodes: Vec<usize>,

    current_character: Option<UserCharacter>,
    last_save_time: std::time::Instant,

    user_config: UserConfig,

    /// Mapped controls from self.user_config
    controls: HashMap<String, egui::Key>,
}
impl Default for TreeVis {
    fn default() -> Self {
        Self {
            camera: RefCell::new((0.0, 0.0)),
            zoom: 0.20,
            passive_tree: PassiveTree::default(),
            hovered_node: None,
            path_nodes: Vec::new(),
            current_character: None,
            last_save_time: std::time::Instant::now(),
            active_edges: HashSet::new(),
            color_map: HashMap::new(),
            fuzzy_search_open: AtomicBool::new(false),
            search_query: String::new(),
            search_results: Vec::new(),
            start_node_id: 0,
            target_node_id: 0,
            path: Vec::new(),
            user_config: UserConfig::default(),
            controls: HashMap::new(),
        }
    }
}

impl TreeVis {
    fn enable_fuzzy_search(&self) {
        self.fuzzy_search_open.store(true, Ordering::Relaxed);
    }

    fn disable_fuzzy_search(&self) {
        self.fuzzy_search_open.store(false, Ordering::Relaxed);
    }

    fn is_fuzzy_search_open(&self) -> bool {
        self.fuzzy_search_open.load(Ordering::Relaxed)
    }
    fn camera_x(&self) -> f32 {
        self.camera.borrow().0
    }

    fn camera_y(&self) -> f32 {
        self.camera.borrow().1
    }

    fn camera_xy(&self) -> (f32, f32) {
        *self.camera.borrow()
    }

    fn find_arbitrary_path(&mut self) {
        if self.path_nodes.len() < 2 {
            return; // Need at least two nodes
        }

        let mut full_path = Vec::new();
        for pair in self.path_nodes.windows(2) {
            if let [start, target] = pair {
                let segment = self.passive_tree.find_shortest_path(*start, *target);
                if full_path.is_empty() {
                    full_path.extend(segment);
                } else {
                    full_path.extend(segment.iter().skip(1));
                }
            }
        }
        self.path = full_path;
        self.update_active_edges();
    }

    fn update_active_edges(&mut self) {
        self.active_edges.clear();
        for window in self.path.windows(2) {
            if let [a, b] = window {
                self.active_edges.insert((*a, *b));
                self.active_edges.insert((*b, *a));
            }
        }
    }

    fn find_path(&mut self, start: usize, target: usize) {
        let path = self.passive_tree.find_shortest_path(start, target);

        // Update the active path and edges.
        self.path = path.clone();
        self.active_edges.clear();
        for window in path.windows(2) {
            if let [a, b] = window {
                self.active_edges.insert((*a, *b));
                self.active_edges.insert((*b, *a));
            }
        }
    }

    pub fn new(data: PassiveTree, config: UserConfig, character: Option<UserCharacter>) -> Self {
        Self {
            zoom: 0.2,
            camera: RefCell::new((0.0, 0.0)),
            passive_tree: data,
            current_character: character,
            ..Default::default()
        }
    }

    fn world_to_screen_x(&self, wx: f64) -> f32 {
        ((wx as f32) - self.camera_x()) * self.zoom + 400.0
    }

    fn world_to_screen_y(&self, wy: f64) -> f32 {
        ((wy as f32) - self.camera_y()) * self.zoom + 300.0
    }

    fn screen_to_world_x(&self, sx: f32) -> f64 {
        ((sx - 400.0) / self.zoom + self.camera_x()) as f64
    }

    fn screen_to_world_y(&self, sy: f32) -> f64 {
        ((sy - 300.0) / self.zoom + self.camera_y()) as f64
    }

    fn update_hover(&mut self, mx: f64, my: f64) {
        let search_radius = 10.0; // Adjustable hover radius //TODO: CONST/CONFIG
        let mut best_dist = f64::MAX; //TODO: lazy once cell on the max distance of the two furthest nodes + a delta...
        let mut best_id = None;

        // Iterate over nodes, but filter based on approximate location first
        for (&id, node) in self.passive_tree.nodes.iter() {
            if (node.wx - mx).abs() > search_radius || (node.wy - my).abs() > search_radius {
                continue; // Skip nodes too far
            }
            let dx = node.wx - mx;
            let dy = node.wy - my;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < search_radius && dist < best_dist {
                best_dist = dist;
                best_id = Some(id);
            }
        }
        self.hovered_node = best_id;
    }

    fn move_camera_to_node(&self, node_id: usize) {
        if let Some(node) = self.passive_tree.nodes.get(&node_id) {
            let mut camera = self.camera.borrow_mut();
            camera.0 = node.wx as f32;
            camera.1 = node.wy as f32;
        }
    }
    fn go_to_node(&self, id: usize) {
        self.move_camera_to_node(id);
        self.disable_fuzzy_search();
    }
    fn select_node(&mut self, node_id: usize) {
        if let Some(character) = &mut self.current_character {
            if !character.activated_node_ids.contains(&node_id) {
                character.activated_node_ids.push(node_id);
                self.save_character();
            }
        }
    }

    fn save_character(&mut self) {
        if let Some(character) = &self.current_character {
            character.save_to_toml("data/last_character.toml");
            self.last_save_time = std::time::Instant::now();
        }
    }

    fn auto_save_character(&mut self) {
        if let Some(character) = &self.current_character {
            if self.last_save_time.elapsed().as_secs() >= 5 {
                self.save_character();
            }
        }
    }

    pub fn load_character(&mut self, path: &str) {
        self.current_character = UserCharacter::load_from_toml(path);
    }

    fn highlight_activated_nodes(&self, painter: &egui::Painter) {
        if let Some(character) = &self.current_character {
            for &node_id in &character.activated_node_ids {
                if let Some(node) = self.passive_tree.nodes.get(&node_id) {
                    painter.circle_filled(
                        egui::pos2(
                            self.world_to_screen_x(node.wx),
                            self.world_to_screen_y(node.wy),
                        ),
                        10.0, // slightly larger size
                        parse_color(
                            self.color_map
                                .get("green")
                                .unwrap_or(&"#29D398".to_string()),
                        ),
                    );
                }
            }
        }
    }
}
