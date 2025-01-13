use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{config::parse_color, data::PassiveTree};

impl eframe::App for TreeVis {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Main draw area:
        egui::CentralPanel::default().show(ctx, |ui| {
            let available = ui.available_size();
            let (rect, _resp) = ui.allocate_at_least(available, egui::Sense::drag());
            let painter = ui.painter_at(rect);

            // WASD movement
            let step = 20.0 / self.zoom;
            if ui.input(|i| i.key_down(egui::Key::W)) {
                self.camera.borrow_mut().1 -= step;
            }
            if ui.input(|i| i.key_down(egui::Key::S)) {
                self.camera.borrow_mut().1 += step;
            }
            if ui.input(|i| i.key_down(egui::Key::A)) {
                self.camera.borrow_mut().0 -= step;
            }
            if ui.input(|i| i.key_down(egui::Key::D)) {
                self.camera.borrow_mut().0 += step;
            }

            // Mouse wheel zoom
            let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
            if scroll_delta != 0.0 {
                self.zoom += 0.001 * scroll_delta;
                self.zoom = self.zoom.clamp(0.01, 100.0);
            }

            // Check mouse hover
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
                        self.color_map
                            .get("yellow")
                            .map_or(egui::Color32::GRAY, |col| parse_color(col))
                    } else {
                        self.color_map
                            .get("default")
                            .map_or(egui::Color32::GRAY, |col| parse_color(col))
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
                    self.color_map
                        .get("red")
                        .map_or(egui::Color32::GRAY, |col| parse_color(col))
                } else if node.is_notable {
                    self.color_map
                        .get("yellow")
                        .map_or(egui::Color32::GRAY, |col| parse_color(col))
                } else {
                    self.color_map
                        .get("blue")
                        .map_or(egui::Color32::GRAY, |col| parse_color(col))
                };

                painter.circle_filled(egui::pos2(sx, sy), node_size, color);
            }

            // Hover text
            if let Some(id) = self.hovered_node {
                if let Some(node) = self.passive_tree.nodes.get(&id) {
                    let sx = self.world_to_screen_x(node.wx) + rect.min.x;
                    let sy = self.world_to_screen_y(node.wy) + rect.min.y;
                    let info_text =
                        format!("\nID:{}\n{}\n{:?}", node.node_id, node.name, node.stats);
                    painter.text(
                        egui::pos2(sx + 10.0, sy - 10.0),
                        egui::Align2::LEFT_TOP,
                        info_text,
                        egui::FontId::default(),
                        self.color_map
                            .get("foreground")
                            .map_or(egui::Color32::WHITE, |col| parse_color(col)),
                    );
                }
            }
        });

        // Zoom slider at bottom panel
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Zoom:");
                ui.add(egui::Slider::new(&mut self.zoom, 0.01..=100.0));
            });
        });

        // Fuzzy search handling
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

        // Path Finder UI
        egui::SidePanel::right("path_panel").show(ctx, |ui| {
            ui.heading("Path Finder");
            ui.label("Start Node:");
            ui.add(egui::DragValue::new(&mut self.start_node_id));
            ui.label("Target Node:");
            ui.add(egui::DragValue::new(&mut self.target_node_id));

            if ui.button("Find Path").clicked() {
                self.find_path(self.start_node_id, self.target_node_id);
            }

            egui::CollapsingHeader::new("Path").show(ui, |ui| {
                for &pid in &self.path {
                    if let Some(node) = self.passive_tree.nodes.get(&pid) {
                        ui.label(format!("ID: {} - {}", pid, node.name));
                    } else {
                        ui.label(format!("Unknown Node ID: {}", pid));
                    }
                }
            });
        });

        // Debug overlay for camera info
        egui::Window::new("Camera info")
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::Vec2::new(-10.0, -10.0))
            .collapsible(false)
            .resizable(false)
            .title_bar(false)
            .show(ctx, |ui| {
                let (dx, dy) = self.camera_xy();
                let dist = (dx * dx + dy * dy).sqrt();
                ui.label(format!(
                    "pos: ({:.2}, {:.2})\nzoom: {:.2}\ndist: {:.2}",
                    dx, dy, self.zoom, dist
                ));
            });
    }
}

#[derive(Default)]
pub struct TreeVis {
    camera: RefCell<(f32, f32)>,
    zoom: f32,
    passive_tree: crate::data::PassiveTree,
    hovered_node: Option<usize>,

    // 1) Fuzzy-search-related
    fuzzy_search_open: AtomicBool,
    search_query: String,
    search_results: Vec<usize>,

    // 2) Path-finder-related
    start_node_id: usize,
    target_node_id: usize,
    path: Vec<usize>,

    /// Store edges of the current path
    active_edges: HashSet<(usize, usize)>,

    // 3) Config-driven colours
    color_map: HashMap<String, String>,
}
// Pointery/Threaddy/Atomicy helpers:
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

    pub fn new(mut data: PassiveTree, color_map: HashMap<String, String>) -> Self {
        data.compute_positions_and_stats();
        Self {
            zoom: 0.2,
            camera: RefCell::new((0.0, 0.0)),
            passive_tree: data,
            color_map,
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
}
