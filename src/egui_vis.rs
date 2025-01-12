use data::{Node, PassiveTree, TreeData};

/// Our eframe app data
struct MyApp {
    camera_x: f64,
    camera_y: f64,
    zoom: f64,
    data: TreeData,
    hovered_node: Option<usize>,
}

impl MyApp {
    fn new(data: TreeData) -> Self {
        Self {
            camera_x: 0.0,
            camera_y: 0.0,
            zoom: 1.0,
            data,
            hovered_node: None,
        }
    }

    fn screen_to_world_x(&self, sx: f32) -> f64 {
        (sx as f64 - 400.0) / self.zoom + self.camera_x
    }
    fn screen_to_world_y(&self, sy: f32) -> f64 {
        (sy as f64 - 300.0) / self.zoom + self.camera_y
    }
    fn world_to_screen_x(&self, wx: f64) -> f64 {
        (wx - self.camera_x) * self.zoom + 400.0
    }
    fn world_to_screen_y(&self, wy: f64) -> f64 {
        (wy - self.camera_y) * self.zoom + 300.0
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
}

/// Implement `eframe::App` for our struct
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let avail = ui.available_size();
            let (rect, _resp) = ui.allocate_at_least(avail, egui::Sense::drag());
            let painter = ui.painter_at(rect);

            // W/A/S/D
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

            // Mouse scroll => zoom
            let scroll_delta = ui.input(|i| i.scroll_delta.y);
            if scroll_delta != 0.0 {
                self.zoom += 0.1 * scroll_delta as f64;
                if self.zoom < 0.1 {
                    self.zoom = 0.1;
                }
                if self.zoom > 100.0 {
                    self.zoom = 100.0;
                }
            }

            // Hover & click
            if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                if rect.contains(pos) {
                    let mx = self.screen_to_world_x(pos.x - rect.min.x);
                    let my = self.screen_to_world_y(pos.y - rect.min.y);
                    self.update_hover(mx, my);

                    // left-click => toggle active
                    if ui.input(|i| i.pointer.primary_clicked()) {
                        if let Some(h) = self.hovered_node {
                            if let Some(n) = self.data.passive_tree.nodes.get_mut(&h) {
                                n.active = !n.active;
                            }
                        }
                    }
                }
            }

            // Draw edges
            for node in self.data.passive_tree.nodes.values() {
                for &c in &node.connections {
                    if let Some(other) = self.data.passive_tree.nodes.get(&c) {
                        let (sx1, sy1) = (
                            self.world_to_screen_x(node.wx) + rect.min.x,
                            self.world_to_screen_y(node.wy) + rect.min.y,
                        );
                        let (sx2, sy2) = (
                            self.world_to_screen_x(other.wx) + rect.min.x,
                            self.world_to_screen_y(other.wy) + rect.min.y,
                        );
                        painter.line_segment(
                            [
                                egui::pos2(sx1 as f32, sy1 as f32),
                                egui::pos2(sx2 as f32, sy2 as f32),
                            ],
                            egui::Stroke::new(2.0, egui::Color32::LIGHT_GRAY),
                        );
                    }
                }
            }

            // Draw nodes
            let node_size = 6.0;
            for (id, node) in &self.data.passive_tree.nodes {
                let sx = self.world_to_screen_x(node.wx) + rect.min.x;
                let sy = self.world_to_screen_y(node.wy) + rect.min.y;
                let color = if node.active {
                    egui::Color32::RED
                } else if node.is_notable {
                    egui::Color32::YELLOW
                } else {
                    egui::Color32::BLUE
                };
                painter.circle_filled(egui::pos2(sx as f32, sy as f32), node_size, color);
            }

            // Hover text
            if let Some(h) = self.hovered_node {
                if let Some(node) = self.data.passive_tree.nodes.get(&h) {
                    let sx = self.world_to_screen_x(node.wx) + rect.min.x;
                    let sy = self.world_to_screen_y(node.wy) + rect.min.y;
                    let text = format!("{}\n{:?}", node.name, node.stats);
                    painter.text(
                        egui::pos2((sx + 10.0) as f32, (sy - 10.0) as f32),
                        egui::Align2::LEFT_TOP,
                        text,
                        egui::FontId::default(),
                        egui::Color32::WHITE,
                    );
                }
            }
        });
    }
}

/// Main entry
fn main() {
    // Create some dummy data:
    let mut data = TreeData {
        passive_tree: PassiveTree {
            groups: HashMap::new(),
            nodes: HashMap::new(),
        },
        passive_skills: HashMap::new(),
    };

    data.passive_tree.nodes.insert(
        0,
        data::Node {
            skill_id: None,
            parent: 0,
            radius: 0,
            position: 0,
            connections: vec![1],
            name: "Center Node".to_string(),
            is_notable: true,
            stats: vec![("bonus".to_string(), 12.0)],
            wx: 0.0,
            wy: 0.0,
            active: false,
        },
    );
    data.passive_tree.nodes.insert(
        1,
        data::Node {
            skill_id: None,
            parent: 0,
            radius: 0,
            position: 0,
            connections: vec![0],
            name: "Second Node".to_string(),
            is_notable: false,
            stats: vec![("life".to_string(), 5.0)],
            wx: 150.0,
            wy: 0.0,
            active: false,
        },
    );

    let native_options = eframe::NativeOptions::default();

    // Start the app, in eframe 0.30 style:
    eframe::run_native(
        "Egui + data.rs (0.30)",
        native_options,
        Box::new(|_cc| Box::new(MyApp::new(data))),
    );
}
