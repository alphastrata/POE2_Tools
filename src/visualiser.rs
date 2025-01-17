//$ src/visualiser.rs
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    default::Default,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::config::{parse_color, UserCharacter};
use crate::{config::UserConfig, data::poe_tree::PassiveTree};

impl eframe::App for TreeVis<'_> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_fuzzy_search(ctx);
        // self.update_hover_effects(ctx);
        // self.update_node_selection(ctx);
        // self.handle_keyboard_input(ctx);
        self.redraw_tree(ctx);
    }
}

// Helper Functions
impl TreeVis<'_> {
    fn update_fuzzy_search(&mut self, ctx: &egui::Context) {
        if self.is_fuzzy_search_open() {
            egui::Window::new("Fuzzy Search")
                .collapsible(true)
                .show(ctx, |ui| {
                    let response = ui.text_edit_singleline(&mut self.search_query);
                    if response.changed() {
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

    // fn update_hover_effects(&mut self, ctx: &egui::Context) {
    //     if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
    //         let mx: f32 = self.screen_to_world_x(pos.x).into();
    //         let my: f32 = self.screen_to_world_y(pos.y).into();
    //         self.update_hover(mx, my);

    //         if let Some(hovered_id) = self.hovered_node {
    //             self.passive_tree.get_edges().iter().for_each(|edge| {
    //                 if edge.source == hovered_id || edge.target == hovered_id {
    //                     self.active_edges.insert((edge.source, edge.target));
    //                 }
    //             });
    //         }
    //     }
    // }

    // fn update_node_selection(&mut self, ctx: &egui::Context) {
    //     if ctx.input(|i| i.pointer.primary_clicked()) {
    //         if let Some(node_id) = self.hovered_node {
    //             self.toggle_node_selection(node_id);
    //         }
    //     }
    // }

    fn redraw_tree(&self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();
            for node in self.passive_tree.nodes.values() {
                let sx = self.world_to_screen_x(node.wx);
                let sy = self.world_to_screen_y(node.wy);
                let radius = if node.active { 12.0 } else { 8.0 };

                let color = if node.active {
                    egui::Color32::from_rgb(0, 255, 0)
                } else {
                    egui::Color32::from_rgb(200, 200, 200)
                };

                painter.circle_filled(egui::pos2(sx, sy), radius, color);
            }

            // for (source_id, target_id) in self.passive_tree.get_edges() {
            //     if let (Some(src), Some(tgt)) = (
            //         self.passive_tree.nodes.get(&source_id),
            //         self.passive_tree.nodes.get(&target_id),
            //     ) {
            //         let sx = self.world_to_screen_x(src.wx);
            //         let sy = self.world_to_screen_y(src.wy);
            //         let tx = self.world_to_screen_x(tgt.wx);
            //         let ty = self.world_to_screen_y(tgt.wy);

            //         painter.line_segment(
            //             [egui::pos2(sx, sy), egui::pos2(tx, ty)],
            //             egui::Stroke::new(1.0, egui::Color32::from_rgb(150, 150, 150)),
            //         );
            //     }
            // }
        });
    }

    // fn toggle_node_selection(&mut self, node_id: usize) {
    //     if let Some(character) = &mut self.current_character {
    //         if character.activated_node_ids.contains(&node_id) {
    //             character.activated_node_ids.retain(|&nid| nid != node_id);
    //         } else {
    //             character.activated_node_ids.push(node_id);
    //         }
    //     }
    //     if let Some(node) = self.passive_tree.nodes.get_mut(&node_id) {
    //         node.active = !node.active;
    //     }
    // }

    fn update_hover(&mut self, mx: f32, my: f32) {
        self.hovered_node = self
            .passive_tree
            .nodes
            .iter()
            .find(|(_, node)| {
                let dx = mx - node.wx as f32;
                let dy = my - node.wy as f32;
                (dx * dx + dy * dy).sqrt() < 10.0 // Hover threshold
            })
            .map(|(id, _)| *id);
    }
}

pub struct TreeVis<'p> {
    camera: RefCell<(f32, f32)>,
    zoom: f32,
    passive_tree: &'p PassiveTree,
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

impl TreeVis<'_> {
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
        todo!()
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

    // fn update_hover(&mut self, mx: f64, my: f64) {
    //     let search_radius = 10.0; // Adjustable hover radius //TODO: CONST/CONFIG
    //     let mut best_dist = f64::MAX; //TODO: lazy once cell on the max distance of the two furthest nodes + a delta...
    //     let mut best_id = None;

    //     // Iterate over nodes, but filter based on approximate location first
    //     for (&id, node) in self.passive_tree.nodes.iter() {
    //         if (node.wx - mx).abs() > search_radius || (node.wy - my).abs() > search_radius {
    //             continue; // Skip nodes too far
    //         }
    //         let dx = node.wx - mx;
    //         let dy = node.wy - my;
    //         let dist = (dx * dx + dy * dy).sqrt();
    //         if dist < search_radius && dist < best_dist {
    //             best_dist = dist;
    //             best_id = Some(id);
    //         }
    //     }
    //     self.hovered_node = best_id;
    // }

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
