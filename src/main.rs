use ggez::{
    conf, event::{self, EventHandler}, glam::Vec2, graphics::{
        Canvas, Color, DrawMode, DrawParam, Mesh, Rect, Text, TextFragment,
    }, input::keyboard::{KeyCode, KeyInput}, Context, ContextBuilder, GameResult
};
use serde_json::Value;
use std::{collections::HashMap, f32::consts::PI, fs};

// Example orbit info; adjust to your data
const ORBIT_RADII: [f32; 8] = [0.0, 82.0, 162.0, 335.0, 493.0, 662.0, 812.0, 972.0];
const ORBIT_SLOTS: [usize; 8] = [1, 6, 16, 16, 40, 60, 60, 60];

// -----------------------------------------------------------------------------
// Data structures
// -----------------------------------------------------------------------------
#[derive(Debug, Clone)]
struct Group {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone)]
struct Node {
    skill_id: Option<String>,
    parent: i64,    // group ID
    radius: i64,    // orbit index
    position: i64,  // orbit slot
    connections: Vec<i64>,

    // Derived fields for rendering:
    name: String,
    is_notable: bool,
    stats: Vec<(String, f32)>,
    // Computed world coords
    wx: f32,
    wy: f32,
}

#[derive(Debug, Clone)]
struct PassiveSkill {
    name: Option<String>,
    is_notable: bool,
    stats: HashMap<String, f32>,
}

#[derive(Debug, Clone)]
struct PassiveTree {
    groups: HashMap<i64, Group>,
    nodes: HashMap<i64, Node>,
}

#[derive(Debug, Clone)]
struct TreeData {
    passive_tree: PassiveTree,
    passive_skills: HashMap<String, PassiveSkill>,
}

// A simple camera
#[derive(Debug, Clone)]
struct Camera {
    pos: Vec2,
    zoom: f32,
}

// -----------------------------------------------------------------------------
// Main visualisation state
// -----------------------------------------------------------------------------
struct TreeVisualization {
    data: TreeData,
    camera: Camera,
    hovered_node: Option<i64>,
}

impl TreeVisualization {
    fn new(mut data: TreeData) -> Self {
        let mut vis = Self {
            data,
            camera: Camera {
                pos: Vec2::new(0.0, 0.0),
                zoom: 1.0,
            },
            hovered_node: None,
        };
        vis.compute_positions_and_stats();
        vis
    }

    /// Convert screen coords to world coords (manual inverse transform).
    /// We treat the centre of the screen as the camera pivot.
    fn screen_to_world(&self, sx: f32, sy: f32, screen_w: f32, screen_h: f32) -> (f32, f32) {
        let cx = screen_w * 0.5;
        let cy = screen_h * 0.5;
        let wx = (sx - cx) / self.camera.zoom + self.camera.pos.x;
        let wy = (sy - cy) / self.camera.zoom + self.camera.pos.y;
        (wx, wy)
    }

    /// Fill each node's (wx, wy), plus name/stats from the skill table.
    fn compute_positions_and_stats(&mut self) {
        let node_ids: Vec<i64> = self.data.passive_tree.nodes.keys().copied().collect();
        for nid in node_ids {
            if let Some(mut node) = self.data.passive_tree.nodes.get(&nid).cloned() {
                // group pos
                if let Some(g) = self.data.passive_tree.groups.get(&node.parent) {
                    let r = ORBIT_RADII
                        .get(node.radius as usize)
                        .copied()
                        .unwrap_or(0.0);
                    let slots = ORBIT_SLOTS
                        .get(node.radius as usize)
                        .copied()
                        .unwrap_or(1) as f32;
                    let angle = node.position as f32 * (2.0 * PI / slots);

                    node.wx = g.x + r * angle.cos();
                    node.wy = g.y + r * angle.sin();
                }
                // fill skill data
                if let Some(sid) = &node.skill_id {
                    if let Some(skill) = self.data.passive_skills.get(sid) {
                        node.name = skill.name.clone().unwrap_or_default();
                        node.is_notable = skill.is_notable;
                        node.stats = skill
                            .stats
                            .iter()
                            .map(|(k, &v)| (k.clone(), v))
                            .collect();
                    }
                }
                self.data.passive_tree.nodes.insert(nid, node);
            }
        }
    }

    /// Check which node, if any, is hovered
    fn update_hover(&mut self, mx: f32, my: f32) {
        let mut best_dist = f32::MAX;
        let mut best_id = None;
        for (&id, node) in &self.data.passive_tree.nodes {
            let dx = node.wx - mx;
            let dy = node.wy - my;
            let dist = (dx * dx + dy * dy).sqrt();
            // 10.0 is a hover threshold
            if dist < 10.0 && dist < best_dist {
                best_dist = dist;
                best_id = Some(id);
            }
        }
        self.hovered_node = best_id;
    }
}

// -----------------------------------------------------------------------------
// EventHandler
// -----------------------------------------------------------------------------
impl EventHandler for TreeVisualization {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // Create a canvas for rendering
        let mut canvas = Canvas::from_frame(ctx, Color::BLACK);

        // We'll get the screen size from the context
        let (screen_w, screen_h) = ctx.gfx.drawable_size();

        // Draw edges
        for (_id, node) in &self.data.passive_tree.nodes {
            for cid in &node.connections {
                if let Some(other) = self.data.passive_tree.nodes.get(cid) {
                    let (sx1, sy1) = world_to_screen(
                        node.wx,
                        node.wy,
                        self.camera.pos.x,
                        self.camera.pos.y,
                        self.camera.zoom,
                        screen_w,
                        screen_h,
                    );
                    let (sx2, sy2) = world_to_screen(
                        other.wx,
                        other.wy,
                        self.camera.pos.x,
                        self.camera.pos.y,
                        self.camera.zoom,
                        screen_w,
                        screen_h,
                    );
                    let line = Mesh::new_line(
                        ctx,
                        &[[sx1, sy1], [sx2, sy2]],
                        1.0,
                        Color::from_rgb(128, 128, 128),
                    )?;
                    canvas.draw(&line, DrawParam::default());
                }
            }
        }

        // Draw nodes
        for (_id, node) in &self.data.passive_tree.nodes {
            let (sx, sy) = world_to_screen(
                node.wx,
                node.wy,
                self.camera.pos.x,
                self.camera.pos.y,
                self.camera.zoom,
                screen_w,
                screen_h,
            );
            let color = if node.is_notable {
                Color::YELLOW
            } else {
                Color::BLUE
            };
            let radius = if node.is_notable { 6.0 } else { 3.0 };
            let circle = Mesh::new_circle(ctx, DrawMode::fill(), [sx, sy], radius, 0.1, color)?;
            canvas.draw(&circle, DrawParam::default());
        }

        // If we're hovering a node, draw a tooltip
        if let Some(id) = self.hovered_node {
            if let Some(node) = self.data.passive_tree.nodes.get(&id) {
                let (sx, sy) = world_to_screen(
                    node.wx,
                    node.wy,
                    self.camera.pos.x,
                    self.camera.pos.y,
                    self.camera.zoom,
                    screen_w,
                    screen_h,
                );
                let stats_text = node
                    .stats
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect::<Vec<_>>()
                    .join("\n");
                let content = format!("{}\n{}", node.name, stats_text);

                let text = Text::new(TextFragment::new(content).color(Color::WHITE));
                let dim = text.measure(ctx)?;
                let bg_rect = Rect::new(sx + 8.0, sy - 4.0, dim.x + 8.0, dim.y + 8.0);
                let bg = Mesh::new_rectangle(ctx, DrawMode::fill(), bg_rect, Color::from_rgba(0, 0, 0, 180))?;
                canvas.draw(&bg, DrawParam::default());
                canvas.draw(&text, DrawParam::default().dest([sx + 12.0, sy]));
            }
        }

        // Present it
        canvas.finish(ctx)
    }

    // In ggez 0.9, this signature is:
    // fn mouse_wheel_event(&mut self, ctx: &mut Context, x: f32, y: f32) -> GameResult
    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, y: f32) -> GameResult {
        // mouse wheel up => y>0 => zoom in
        self.camera.zoom += 0.1 * y;
        if self.camera.zoom < 0.1 {
            self.camera.zoom = 0.1;
        }
        if self.camera.zoom > 100.0 {
            self.camera.zoom = 100.0;
        }
        Ok(())
    }

    // In ggez 0.9, this signature is:
    // fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, dx: f32, dy: f32) -> GameResult
    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) -> GameResult {
        let (screen_w, screen_h) = ctx.gfx.drawable_size();
        let (wx, wy) = self.screen_to_world(x, y, screen_w, screen_h);
        self.update_hover(wx, wy);
        Ok(())
    }

    // fn key_down_event(&mut self, ctx: &mut Context, input: KeyInput, _repeated: bool) -> GameResult {
    //     if let Some(key) = input.keycode {
    //         let step = 50.0 / self.camera.zoom;
    //         match key {
    //             KeyCode::W => self.camera.pos.y -= step,
    //             KeyCode::S => self.camera.pos.y += step,
    //             KeyCode::A => self.camera.pos.x -= step,
    //             KeyCode::D => self.camera.pos.x += step,
    //             KeyCode::Escape => {
    //                 ctx.request_quit();
    //             }
    //             _ => {}
    //         }
    //     }
    //     Ok(())
    // }
    fn key_down_event(&mut self, ctx: &mut Context, input: KeyInput, _repeated: bool) -> GameResult {
        if let Some(key) = input.keycode {
            match key {
                KeyCode::H => {
                    // find node named "Flow Like Water"
                    for node in self.data.passive_tree.nodes.values() {
                        if node.name == "Flow Like Water" {
                            self.camera.pos = Vec2::new(node.wx, node.wy);
                            break;
                        }
                    }
                }
                KeyCode::W => self.camera.pos.y -= 50.0 / self.camera.zoom,
                KeyCode::S => self.camera.pos.y += 50.0 / self.camera.zoom,
                KeyCode::A => self.camera.pos.x -= 50.0 / self.camera.zoom,
                KeyCode::D => self.camera.pos.x += 50.0 / self.camera.zoom,
                KeyCode::Escape => ctx.request_quit(),
                _ => {}
            }
        }
        Ok(())
    }
    
}

// -----------------------------------------------------------------------------
// Helper: world -> screen
// -----------------------------------------------------------------------------
fn world_to_screen(
    wx: f32,
    wy: f32,
    camx: f32,
    camy: f32,
    zoom: f32,
    screen_w: f32,
    screen_h: f32,
) -> (f32, f32) {
    let cx = screen_w * 0.5;
    let cy = screen_h * 0.5;
    let sx = (wx - camx) * zoom + cx;
    let sy = (wy - camy) * zoom + cy;
    (sx, sy)
}

// -----------------------------------------------------------------------------
// JSON loading
// -----------------------------------------------------------------------------
fn load_tree(path: &str) -> TreeData {
    let data = fs::read_to_string(path).expect("Could not read tree file");
    let json: Value = serde_json::from_str(&data).expect("JSON parse error");

    let tree_obj = &json["passive_tree"];
    let mut groups = HashMap::new();
    let mut nodes = HashMap::new();
    let mut skills = HashMap::new();

    // groups
    if let Some(g) = tree_obj["groups"].as_object() {
        for (gid, gval) in g {
            let gx = gval["x"].as_f64().unwrap_or(0.0) as f32;
            let gy = gval["y"].as_f64().unwrap_or(0.0) as f32;
            groups.insert(gid.parse::<i64>().unwrap_or(0), Group { x: gx, y: gy });
        }
    }

    // nodes
    if let Some(n) = tree_obj["nodes"].as_object() {
        for (nid, nval) in n {
            let skill_id = nval["skill_id"].as_str().map(|s| s.to_owned());
            let parent = nval["parent"].as_i64().unwrap_or(0);
            let radius = nval["radius"].as_i64().unwrap_or(0);
            let position = nval["position"].as_i64().unwrap_or(0);

            let connections = nval["connections"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|c| c["id"].as_i64())
                .collect::<Vec<_>>();

            nodes.insert(
                nid.parse::<i64>().unwrap_or(0),
                Node {
                    skill_id,
                    parent,
                    radius,
                    position,
                    connections,
                    name: String::new(),
                    is_notable: false,
                    stats: Vec::new(),
                    wx: 0.0,
                    wy: 0.0,
                },
            );
        }
    }

    // passive_skills
    if let Some(sk_obj) = json["passive_skills"].as_object() {
        for (skill_id, skill_val) in sk_obj {
            let name = skill_val["name"].as_str().map(|s| s.to_string());
            let is_notable = skill_val["is_notable"].as_bool().unwrap_or(false);
            let mut stats = HashMap::new();
            if let Some(st) = skill_val["stats"].as_object() {
                for (k, v) in st {
                    if let Some(num) = v.as_f64() {
                        stats.insert(k.clone(), num as f32);
                    }
                }
            }
            skills.insert(
                skill_id.to_owned(),
                PassiveSkill {
                    name,
                    is_notable,
                    stats,
                },
            );
        }
    }

    TreeData {
        passive_tree: PassiveTree { groups, nodes },
        passive_skills: skills,
    }
}

// -----------------------------------------------------------------------------
// main()
// -----------------------------------------------------------------------------
fn main() -> GameResult {
    let data = load_tree("POE2_TREE.json");

    let (ctx, event_loop) = ContextBuilder::new("passive_tree", "ggez")
        .window_setup(conf::WindowSetup::default().title("Path of Exile Passive Tree"))
        .window_mode(conf::WindowMode::default().dimensions(1920.0, 1080.0))
        .build()?;

    let vis = TreeVisualization::new(data);
    event::run(ctx, event_loop, vis)
}
