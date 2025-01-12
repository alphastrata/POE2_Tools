// src/main.rs
use eframe::egui;
use eframe::epi;

use data::*;

struct Camera {
    pos_x: f32,
    pos_y: f32,
    zoom: f64,
}

struct MyApp {
    data: TreeData,
    camera: Camera,
    hovered_node: Option<usize>,
    // bounding box for clamping:
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
}

impl MyApp {
    fn new(mut data: TreeData) -> Self {
        // Suppose we compute node positions from group + radius + position
        for node in data.passive_tree.nodes.values_mut() {
            if let Some(g) = data.passive_tree.groups.get(&node.parent) {
                // Example orbit logic
                let r = node.radius as f64 * 100.0; // or your custom logic
                let slots = 16.0;
                let angle = node.position as f64 * (2.0 * PI / slots);
                node.wx = g.x + r * angle.cos();
                node.wy = g.y + r * angle.sin();
            }
            // fill from skill
            if let Some(sid) = &node.skill_id {
                if let Some(skill) = data.passive_skills.get(sid) {
                    node.name = skill.name.clone().unwrap_or_default();
                    node.is_notable = skill.is_notable;
                    node.stats = skill.stats.clone();
                }
            }
            node.active = false;
        }

        // find bounding box
        let mut min_x = f64::MAX;
        let mut max_x = f64::MIN;
        let mut min_y = f64::MAX;
        let mut max_y = f64::MIN;
        for n in data.passive_tree.nodes.values() {
            if n.wx < min_x {
                min_x = n.wx;
            }
            if n.wx > max_x {
                max_x = n.wx;
            }
            if n.wy < min_y {
                min_y = n.wy;
            }
            if n.wy > max_y {
                max_y = n.wy;
            }
        }

        Self {
            data,
            camera: Camera {
                pos_x: 0.0,
                pos_y: 0.0,
                zoom: 1.0,
            },
            hovered_node: None,
            min_x,
            max_x,
            min_y,
            max_y,
        }
    }

    fn clamp_camera(&mut self, screen_w: f32, screen_h: f32) {
        // bounding box center cannot go out of view
        let halfw = screen_w * 0.5 / self.camera.zoom as f32;
        let halfh = screen_h * 0.5 / self.camera.zoom as f32;
        let (minx, maxx) = (self.min_x as f32, self.max_x as f32);
        let (miny, maxy) = (self.min_y as f32, self.max_y as f32);
        if self.camera.pos_x < minx + halfw {
            self.camera.pos_x = minx + halfw;
        }
        if self.camera.pos_x > maxx - halfw {
            self.camera.pos_x = maxx - halfw;
        }
        if self.camera.pos_y < miny + halfh {
            self.camera.pos_y = miny + halfh;
        }
        if self.camera.pos_y > maxy - halfh {
            self.camera.pos_y = maxy - halfh;
        }
    }

    // from world->screen
    fn world_to_screen(
        &self,
        world_x: f64,
        world_y: f64,
        screen_w: f32,
        screen_h: f32,
    ) -> (f32, f32) {
        let cx = screen_w * 0.5;
        let cy = screen_h * 0.5;
        let sx = ((world_x as f32 - self.camera.pos_x) * self.camera.zoom as f32) + cx;
        let sy = ((world_y as f32 - self.camera.pos_y) * self.camera.zoom as f32) + cy;
        (sx, sy)
    }

    // from screen->world
    fn screen_to_world(&self, sx: f32, sy: f32, screen_w: f32, screen_h: f32) -> (f64, f64) {
        let cx = screen_w * 0.5;
        let cy = screen_h * 0.5;
        let wx = (sx - cx) / self.camera.zoom as f32 + self.camera.pos_x;
        let wy = (sy - cy) / self.camera.zoom as f32 + self.camera.pos_y;
        (wx as f64, wy as f64)
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

impl epi::App for MyApp {
    fn name(&self) -> &str {
        "Egui + old GGEZ backend"
    }

    fn update(&mut self, ctx: &egui::CtxRef, _frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let size = ui.available_size();
            let (rect, _resp) = ui.allocate_at_least(size, egui::Sense::drag());
            let painter = ui.painter_at(rect);

            // camera movement with WASD (like old ggez)
            let step = 50.0 / self.camera.zoom as f32;
            if ui.input().key_down(egui::Key::W) {
                self.camera.pos_y -= step;
            }
            if ui.input().key_down(egui::Key::S) {
                self.camera.pos_y += step;
            }
            if ui.input().key_down(egui::Key::A) {
                self.camera.pos_x -= step;
            }
            if ui.input().key_down(egui::Key::D) {
                self.camera.pos_x += step;
            }

            // Zoom with scroll
            if ui.input().scroll_delta.y != 0.0 {
                self.camera.zoom += 0.1 * ui.input().scroll_delta.y as f64;
                if self.camera.zoom < 0.1 {
                    self.camera.zoom = 0.1;
                }
                if self.camera.zoom > 100.0 {
                    self.camera.zoom = 100.0;
                }
            }

            // clamp camera
            self.clamp_camera(rect.width(), rect.height());

            // Hover/click
            if let Some(cursor) = ui.input().pointer.hover_pos() {
                if rect.contains(cursor) {
                    // screen->world
                    let (mx, my) = self.screen_to_world(
                        cursor.x - rect.min.x,
                        cursor.y - rect.min.y,
                        rect.width(),
                        rect.height(),
                    );
                    self.update_hover(mx, my);

                    // left click => toggle active
                    if ui.input().pointer.primary_clicked() {
                        if let Some(nid) = self.hovered_node {
                            if let Some(node) = self.data.passive_tree.nodes.get_mut(&nid) {
                                node.active = !node.active;
                            }
                        }
                    }
                }
            }

            // draw edges
            for node in self.data.passive_tree.nodes.values() {
                for cid in &node.connections {
                    if let Some(other) = self.data.passive_tree.nodes.get(cid) {
                        let (sx1, sy1) =
                            self.world_to_screen(node.wx, node.wy, rect.width(), rect.height());
                        let (sx2, sy2) =
                            self.world_to_screen(other.wx, other.wy, rect.width(), rect.height());
                        let x1 = sx1 + rect.min.x;
                        let y1 = sy1 + rect.min.y;
                        let x2 = sx2 + rect.min.x;
                        let y2 = sy2 + rect.min.y;
                        painter.line_segment(
                            [egui::pos2(x1, y1), egui::pos2(x2, y2)],
                            egui::Stroke::new(2.0, egui::Color32::GRAY),
                        );
                    }
                }
            }

            // draw nodes
            let radius = 6.0; // fixed screen radius
            for (id, node) in &self.data.passive_tree.nodes {
                let (sx, sy) = self.world_to_screen(node.wx, node.wy, rect.width(), rect.height());
                let px = sx + rect.min.x;
                let py = sy + rect.min.y;
                let color = if node.active {
                    egui::Color32::RED
                } else if node.is_notable {
                    egui::Color32::YELLOW
                } else {
                    egui::Color32::BLUE
                };
                painter.circle_filled(egui::pos2(px, py), radius, color);
            }

            // hover text
            if let Some(nid) = self.hovered_node {
                if let Some(node) = self.data.passive_tree.nodes.get(&nid) {
                    let (sx, sy) =
                        self.world_to_screen(node.wx, node.wy, rect.width(), rect.height());
                    let px = sx + rect.min.x;
                    let py = sy + rect.min.y;
                    let text = format!("{}\n{:?}", node.name, node.stats);
                    painter.text(
                        egui::pos2(px + 10.0, py - 10.0),
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

fn main() {
    // For demo, just mock some data:
    let mut data = TreeData {
        passive_tree: PassiveTree {
            groups: HashMap::new(),
            nodes: HashMap::new(),
        },
        passive_skills: HashMap::new(),
    };
    data.passive_tree.groups.insert(0, Group { x: 0.0, y: 0.0 });
    data.passive_tree
        .groups
        .insert(1, Group { x: 200.0, y: 50.0 });

    // Insert some nodes
    data.passive_tree.nodes.insert(
        0,
        Node {
            skill_id: None,
            parent: 0,
            radius: 1,
            position: 0,
            connections: vec![1],
            name: "Center".to_owned(),
            is_notable: true,
            stats: vec![("bonus".to_string(), 10.0)],
            wx: 0.0,
            wy: 0.0,
            active: false,
        },
    );
    data.passive_tree.nodes.insert(
        1,
        Node {
            skill_id: None,
            parent: 1,
            radius: 1,
            position: 4,
            connections: vec![0],
            name: "Linked".to_owned(),
            is_notable: false,
            stats: vec![("life".to_string(), 5.0)],
            wx: 0.0,
            wy: 0.0,
            active: false,
        },
    );

    let app = MyApp::new(data);
    let opts = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), opts);
}
