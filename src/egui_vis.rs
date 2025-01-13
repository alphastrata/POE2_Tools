// In your main.rs (or wherever your main entry is):
use poo_tools::data::PassiveTree;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
struct TreeVis {
    camera_x: f32,
    camera_y: f32,
    zoom: f32,
    data: PassiveTree,
    hovered_node: Option<usize>,

    // 1) Fuzzy-search-related
    fuzzy_search_open: bool,
    search_query: String,
    search_results: Vec<usize>,

    // 2) Path-finder-related
    start_node_id: usize,
    target_node_id: usize,
    path: Vec<usize>,

    // 3) Config-driven colours
    color_map: HashMap<String, String>,

    explored_paths: Vec<Vec<usize>>,
}

impl TreeVis {
    fn find_path(
        tree: &mut PassiveTree,
        explored_paths: &mut Vec<Vec<usize>>,
        start: usize,
        target: usize,
    ) -> Vec<usize> {
        explored_paths.clear();
        tree.find_path_with_limit(start, target, explored_paths)
    }
}

fn path_to_edge_set(path: &[usize]) -> HashSet<(usize, usize)> {
    let mut edges = HashSet::new();
    for w in path.windows(2) {
        edges.insert((w[0], w[1]));
    }
    edges
}

fn parse_color(col_str: &str) -> egui::Color32 {
    match col_str.to_lowercase().as_str() {
        "red" => egui::Color32::RED,
        "green" => egui::Color32::GREEN,
        "blue" => egui::Color32::BLUE,
        "yellow" => egui::Color32::YELLOW,
        "white" => egui::Color32::WHITE,
        _ => egui::Color32::GRAY,
    }
}

//-----------------------------------------------------------------------------------
// TreeVis implementation
//-----------------------------------------------------------------------------------
impl TreeVis {
    fn new(mut data: PassiveTree, color_map: HashMap<String, String>) -> Self {
        data.compute_positions_and_stats();
        Self {
            camera_x: 0.0,
            camera_y: 0.0,
            zoom: 1.0,
            data,
            hovered_node: None,
            fuzzy_search_open: false,
            search_query: String::new(),
            search_results: Vec::new(),
            start_node_id: 0,
            target_node_id: 0,
            path: Vec::new(),
            color_map,
            explored_paths: Vec::new(),
        }
    }

    fn world_to_screen_x(&self, wx: f64) -> f32 {
        ((wx as f32) - self.camera_x) * self.zoom + 400.0
    }
    fn world_to_screen_y(&self, wy: f64) -> f32 {
        ((wy as f32) - self.camera_y) * self.zoom + 300.0
    }

    fn screen_to_world_x(&self, sx: f32) -> f64 {
        ((sx - 400.0) / self.zoom + self.camera_x as f32) as f64
    }
    fn screen_to_world_y(&self, sy: f32) -> f64 {
        ((sy - 300.0) / self.zoom + self.camera_y as f32) as f64
    }

    fn update_hover(&mut self, mx: f64, my: f64) {
        let mut best_dist = f64::MAX;
        let mut best_id = None;
        for (&id, node) in &self.data.passive_tree.nodes {
            let dx = node.wx - mx;
            let dy = node.wy - my;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < 10.0 && dist < best_dist {
                best_dist = dist;
                best_id = Some(id);
            }
        }
        self.hovered_node = best_id;
    }

    // when user double-clicks a result
    fn go_to_node(&mut self, id: usize) {
        if let Some(node) = self.data.passive_tree.nodes.get(&id) {
            // center camera on node
            self.camera_x = node.wx as f32;
            self.camera_y = node.wy as f32;
        }
        self.fuzzy_search_open = false;
    }
}

//-----------------------------------------------------------------------------------
// eframe::App for TreeVis (where you integrate new UI code)
//-----------------------------------------------------------------------------------
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
                self.camera_y -= step;
            }
            if ui.input(|i| i.key_down(egui::Key::S)) {
                self.camera_y += step;
            }
            if ui.input(|i| i.key_down(egui::Key::A)) {
                self.camera_x -= step;
            }
            if ui.input(|i| i.key_down(egui::Key::D)) {
                self.camera_x += step;
            }

            // mouse wheel zoom
            let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
            if scroll_delta != 0.0 {
                self.zoom += 0.001 * scroll_delta;
                self.zoom = self.zoom.clamp(0.1, 100.0);
            }

            // Check mouse hover
            if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                if rect.contains(pos) {
                    let mx = self.screen_to_world_x(pos.x - rect.min.x);
                    let my = self.screen_to_world_y(pos.y - rect.min.y);
                    self.update_hover(mx, my);
                    // toggle node active on click
                    if ui.input(|i| i.pointer.primary_clicked()) {
                        if let Some(id) = self.hovered_node {
                            if let Some(node) = self.data.passive_tree.nodes.get_mut(&id) {
                                node.active = !node.active;
                            }
                        }
                    }
                }
            }

            // Draw edges
            let path_edges = path_to_edge_set(&self.path);
            for (&nid, node) in &self.data.passive_tree.nodes {
                for &other_id in &node.connections {
                    if let Some(other) = self.data.passive_tree.nodes.get(&other_id) {
                        let sx1 = self.world_to_screen_x(node.wx) + rect.min.x;
                        let sy1 = self.world_to_screen_y(node.wy) + rect.min.y;
                        let sx2 = self.world_to_screen_x(other.wx) + rect.min.x;
                        let sy2 = self.world_to_screen_y(other.wy) + rect.min.y;
                        let is_on_path = path_edges.contains(&(nid, other_id))
                            || path_edges.contains(&(other_id, nid));
                        let stroke = if is_on_path {
                            egui::Stroke::new(3.0, egui::Color32::GREEN)
                        } else {
                            egui::Stroke::new(2.0, egui::Color32::GRAY)
                        };
                        painter.line_segment([egui::pos2(sx1, sy1), egui::pos2(sx2, sy2)], stroke);
                    }
                }
            }

            // Draw nodes
            let base_node_size = 6.0;
            for (id, node) in &self.data.passive_tree.nodes {
                let sx = self.world_to_screen_x(node.wx) + rect.min.x;
                let sy = self.world_to_screen_y(node.wy) + rect.min.y;
                let node_size = base_node_size * (1.0 + self.zoom * 0.1);

                let mut color = if node.active {
                    egui::Color32::RED
                } else if node.is_notable {
                    egui::Color32::YELLOW
                } else {
                    egui::Color32::BLUE
                };

                // Colour by config if stats match keyword
                for (stat, val) in &node.stats {
                    for (kw, col_str) in &self.color_map {
                        if stat.to_lowercase().contains(&kw.to_lowercase()) {
                            color = parse_color(col_str);
                            break;
                        }
                    }
                }

                painter.circle_filled(egui::pos2(sx, sy), node_size, color);
            }

            // Hover text
            if let Some(id) = self.hovered_node {
                if let Some(node) = self.data.passive_tree.nodes.get(&id) {
                    let sx = self.world_to_screen_x(node.wx) + rect.min.x;
                    let sy = self.world_to_screen_y(node.wy) + rect.min.y;
                    let info_text = format!(
                        "\nID:{}\n{}\n{:?}",
                        node.skill_id.clone().unwrap(),
                        node.name,
                        node.stats
                    );
                    painter.text(
                        egui::pos2(sx + 10.0, sy - 10.0),
                        egui::Align2::LEFT_TOP,
                        info_text,
                        egui::FontId::default(),
                        egui::Color32::WHITE,
                    );
                }
            }
        });

        // 1) Moved the zoom slider to a bottom panel so it's visible
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Zoom:");
                ui.add(egui::Slider::new(&mut self.zoom, 0.01..=100.0));
            });
        });

        // 2) 'F' key opens fuzzy search
        if ctx.input(|i| i.key_pressed(egui::Key::F)) {
            self.fuzzy_search_open = true;
        }
        if self.fuzzy_search_open {
            egui::Window::new("Fuzzy Search")
                .collapsible(true)
                .show(ctx, |ui| {
                    let resp = ui.text_edit_singleline(&mut self.search_query);
                    if resp.changed() {
                        self.search_results = self.data.fuzzy_search_nodes(&self.search_query);
                    }
                    egui::CollapsingHeader::new("Results").show(ui, |ui| {
                        for &id in &self.search_results {
                            let node = &self.data.passive_tree.nodes[&id];
                            // Double-click to go to node
                            if ui.selectable_label(false, &node.name).double_clicked() {
                                // self.go_to_node(id);
                                println!("Double clicked {0}, {1:#?}", node.name, node.skill_id)
                            }
                        }
                    });
                });
        }

        // 5) Right panel for start->target path-finder
        egui::SidePanel::right("path_panel").show(ctx, |ui| {
            ui.heading("Path Finder");
            ui.label("Start Node:");
            ui.add(egui::DragValue::new(&mut self.start_node_id));
            ui.label("Target Node:");
            ui.add(egui::DragValue::new(&mut self.target_node_id));
            if ui.button("Find Path").clicked() {
                self.path = Self::find_path(
                    &mut self.data.passive_tree,
                    &mut self.explored_paths,
                    self.start_node_id,
                    self.target_node_id,
                );
            }
            egui::CollapsingHeader::new("Path").show(ui, |ui| {
                for &pid in &self.path {
                    ui.label(format!("Node {}", pid));
                }
            });
        });

        // Optional overlay with camera info
        egui::Window::new("Camera info")
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::Vec2::new(-10.0, -10.0))
            .collapsible(false)
            .resizable(false)
            .title_bar(false)
            .show(ctx, |ui| {
                let dx = self.camera_x;
                let dy = self.camera_y;
                let dist = (dx * dx + dy * dy).sqrt();
                ui.label(format!(
                    "pos: ({:.2}, {:.2})\nzoom: {:.2}\ndist: {:.2}",
                    dx, dy, self.zoom, dist
                ));
            });
    }
}

//-----------------------------------------------------------------------------------
// main (same file or separate)
//-----------------------------------------------------------------------------------
fn main() {
    // Load data
    let mut data = poo_tools::data::PassiveTree::load_tree("data/POE2_TREE.json");
    data.compute_positions_and_stats();

    println!(
        "Found {} nodes and {} groups",
        data.passive_tree.nodes.len(),
        data.passive_tree.groups.len(),
    );

    let config_str = std::fs::read_to_string("tree_config.toml").unwrap();
    let config: toml::Value = toml::from_str(&config_str).unwrap();

    // Build color_map from [colors] table
    let mut color_map = HashMap::new();
    if let Some(colors) = config.get("colors").and_then(|v| v.as_table()) {
        for (k, v) in colors {
            if let Some(col_str) = v.as_str() {
                color_map.insert(k.clone(), col_str.to_string());
            }
        }
    }

    let native_opts = eframe::NativeOptions::default();
    _ = eframe::run_native(
        "Egui + data.rs (f32 fix)",
        native_opts,
        Box::new(|_cc| {
            // pass color_map into TreeVis::new
            Ok(Box::new(TreeVis::new(data, color_map)))
        }),
    );
}
