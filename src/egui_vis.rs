use poo_tools::data::{Node, PassiveSkill, PassiveTree, TreeData};
use std::collections::HashMap;

struct MyApp {
    camera_x: f32, // camera pos in world coords (f32)
    camera_y: f32,
    zoom: f32, // zoom factor
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

    // Convert from world -> screen
    // Our node uses (wx, wy) as f64. We cast to f32 for drawing.
    fn world_to_screen_x(&self, wx: f64) -> f32 {
        ((wx as f32) - self.camera_x) * self.zoom + 400.0
    }
    fn world_to_screen_y(&self, wy: f64) -> f32 {
        ((wy as f32) - self.camera_y) * self.zoom + 300.0
    }

    // Convert from screen -> world
    fn screen_to_world_x(&self, sx: f32) -> f64 {
        ((sx - 400.0) / self.zoom + self.camera_x as f32) as f64
    }
    fn screen_to_world_y(&self, sy: f32) -> f64 {
        ((sy - 300.0) / self.zoom + self.camera_y as f32) as f64
    }

    // Find which node is hovered, if any
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

// -----------------------------------------------------------------------------
// Implement eframe::App for our struct
// -----------------------------------------------------------------------------
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Draw everything in a central panel
        egui::CentralPanel::default().show(ctx, |ui| {
            // We get the space to draw
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

            // Mouse scroll => zoom
            let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
            if scroll_delta != 0.0 {
                self.zoom += 0.1 * scroll_delta;
                self.zoom = self.zoom.clamp(0.1, 100.0);
            }

            // Check mouse hover/click
            if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                if rect.contains(pos) {
                    // Convert screen -> world
                    let mx = self.screen_to_world_x(pos.x - rect.min.x);
                    let my = self.screen_to_world_y(pos.y - rect.min.y);
                    self.update_hover(mx, my);

                    // If user clicks, toggle node active
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
            for node in self.data.passive_tree.nodes.values() {
                for &c in &node.connections {
                    if let Some(other) = self.data.passive_tree.nodes.get(&c) {
                        // Convert node world coords -> screen coords, then offset by rect
                        let sx1 = self.world_to_screen_x(node.wx) + rect.min.x;
                        let sy1 = self.world_to_screen_y(node.wy) + rect.min.y;
                        let sx2 = self.world_to_screen_x(other.wx) + rect.min.x;
                        let sy2 = self.world_to_screen_y(other.wy) + rect.min.y;

                        painter.line_segment(
                            [egui::pos2(sx1, sy1), egui::pos2(sx2, sy2)],
                            egui::Stroke::new(2.0, egui::Color32::GRAY),
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
                painter.circle_filled(egui::pos2(sx, sy), node_size, color);
            }

            // Hover text
            if let Some(id) = self.hovered_node {
                if let Some(node) = self.data.passive_tree.nodes.get(&id) {
                    let sx = self.world_to_screen_x(node.wx) + rect.min.x;
                    let sy = self.world_to_screen_y(node.wy) + rect.min.y;
                    let info_text = format!("{}\n{:?}", node.name, node.stats);
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
    }
}

// -----------------------------------------------------------------------------
// main
// -----------------------------------------------------------------------------
fn main() {
    // Minimal mock data
    let mut data = TreeData {
        passive_tree: PassiveTree {
            groups: HashMap::new(),
            nodes: HashMap::new(),
        },
        passive_skills: HashMap::new(),
    };

    // Insert some nodes
    data.passive_tree.nodes.insert(
        0,
        Node {
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
        Node {
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

    // Launch eframe
    let native_opts = eframe::NativeOptions::default();
    _ = eframe::run_native(
        "Egui + data.rs (f32 fix)",
        native_opts,
        Box::new(|_cc| Ok(Box::new(MyApp::new(data)))),
    );
}
