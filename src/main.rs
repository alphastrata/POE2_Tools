use ggez::{conf, event, graphics::{Canvas, Color, DrawMode, DrawParam, Mesh, Text}, Context, ContextBuilder, GameResult};
use ggez::mint::Point2;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;

// Just some example orbits (you'll want to adapt these to your data)
const ORBIT_RADII: [f32; 8] = [0.0, 82.0, 162.0, 335.0, 493.0, 662.0, 812.0, 972.0];
const ORBIT_SLOTS: [usize; 8] = [1, 6, 16, 16, 40, 60, 60, 60];

// ----------------------------------------------------
// Basic data structs to hold our parsed JSON
// ----------------------------------------------------

#[derive(Debug, Deserialize)]
struct Connection {
    id: i64,
    // radius: i64, // if needed
}

#[derive(Debug, Deserialize)]
struct Node {
    skill_id: Option<String>,
    parent: i64,
    radius: i64,    // orbit
    position: i64,  // orbit index
    connections: Option<Vec<Connection>>,
}

#[derive(Debug)]
struct Group {
    x: f32,
    y: f32,
}

#[derive(Debug)]
struct PassiveSkill {
    name: Option<String>,
    is_notable: bool,
}

#[derive(Debug)]
struct PassiveTree {
    groups: HashMap<i64, Group>,
    nodes: HashMap<i64, Node>,
}

#[derive(Debug)]
struct TreeData {
    passive_tree: PassiveTree,
    passive_skills: HashMap<String, PassiveSkill>,
}

// ----------------------------------------------------
// Free function to load our JSON
// ----------------------------------------------------
fn load_tree(path: &str) -> TreeData {
    let data = fs::read_to_string(path).expect("Could not read tree file");
    let json: Value = serde_json::from_str(&data).expect("JSON parse error");

    let tree_obj = &json["passive_tree"];
    let mut groups = HashMap::new();
    let mut nodes = HashMap::new();
    let mut skills = HashMap::new();

    // Parse groups
    if let Some(obj) = tree_obj["groups"].as_object() {
        for (grp_id, gval) in obj {
            let gx = gval["x"].as_f64().unwrap_or(0.0) as f32;
            let gy = gval["y"].as_f64().unwrap_or(0.0) as f32;
            groups.insert(grp_id.parse::<i64>().unwrap_or(0), Group { x: gx, y: gy });
        }
    }

    // Parse nodes
    if let Some(obj) = tree_obj["nodes"].as_object() {
        for (nid, nval) in obj {
            let parent = nval["parent"].as_i64().unwrap_or(0);
            let radius = nval["radius"].as_i64().unwrap_or(0);
            let position = nval["position"].as_i64().unwrap_or(0);
            let skill_id = nval["skill_id"].as_str().map(|s| s.to_string());

            let mut conns = Vec::new();
            if let Some(arr) = nval["connections"].as_array() {
                for c in arr {
                    if let Some(cid) = c["id"].as_i64() {
                        conns.push(Connection { id: cid });
                    }
                }
            }
            nodes.insert(
                nid.parse::<i64>().unwrap_or(0),
                Node {
                    skill_id,
                    parent,
                    radius,
                    position,
                    connections: Some(conns),
                },
            );
        }
    }

    // Parse passive_skills
    if let Some(sk_obj) = json["passive_skills"].as_object() {
        for (skid, sval) in sk_obj {
            let name = sval["name"].as_str().map(|s| s.to_string());
            let is_notable = sval["is_notable"].as_bool().unwrap_or(false);
            skills.insert(
                skid.clone(),
                PassiveSkill {
                    name,
                    is_notable,
                },
            );
        }
    }

    TreeData {
        passive_tree: PassiveTree { groups, nodes },
        passive_skills: skills,
    }
}

// ----------------------------------------------------
// Visualization code
// ----------------------------------------------------
struct TreeVisualization {
    data: TreeData,
    node_positions: Vec<(f32, f32, String, bool)>, // (x, y, label, is_notable)
    edges: Vec<((f32, f32), (f32, f32))>,
}

impl TreeVisualization {
    fn new(data: TreeData) -> Self {
        let mut node_positions = Vec::new();
        let mut edges = Vec::new();

        for (&node_id, node) in &data.passive_tree.nodes {
            // get group pos
            let group = data.passive_tree.groups.get(&node.parent);
            if group.is_none() {
                continue;
            }
            let gx = group.unwrap().x;
            let gy = group.unwrap().y;

            // clamp radius indexing
            let r = ORBIT_RADII.get(node.radius as usize).copied().unwrap_or(0.0);
            let slots = ORBIT_SLOTS.get(node.radius as usize).copied().unwrap_or(1) as f32;

            let angle = node.position as f32 * (2.0 * std::f32::consts::PI / slots);
            let x = gx + r * angle.cos();
            let y = gy + r * angle.sin();

            let mut label = String::new();
            let mut is_notable = false;

            if let Some(skid) = &node.skill_id {
                if let Some(sk) = data.passive_skills.get(skid) {
                    label = sk.name.clone().unwrap_or_default();
                    is_notable = sk.is_notable;
                }
            }

            node_positions.push((x, y, label, is_notable));

            // edges
            if let Some(connections) = &node.connections {
                for c in connections {
                    let other_id = c.id;
                    // ensure the connected node actually exists
                    if let Some(connected_node) = data.passive_tree.nodes.get(&other_id) {
                        if let Some(parent_group) = data.passive_tree.groups.get(&connected_node.parent) {
                            let rc = ORBIT_RADII
                                .get(connected_node.radius as usize)
                                .copied()
                                .unwrap_or(0.0);
                            let sc = ORBIT_SLOTS
                                .get(connected_node.radius as usize)
                                .copied()
                                .unwrap_or(1) as f32;
                            let angle_c = connected_node.position as f32
                                * (2.0 * std::f32::consts::PI / sc);
                            let cx = parent_group.x + rc * angle_c.cos();
                            let cy = parent_group.y + rc * angle_c.sin();
                            edges.push(((x, y), (cx, cy)));
                        }
                    }
                }
            }
        }

        Self { data, node_positions, edges }
    }
}

impl event::EventHandler for TreeVisualization {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::from_frame(ctx, Color::BLACK);

        // edges
        for &((x1, y1), (x2, y2)) in &self.edges {
            let line = Mesh::new_line(
                ctx,
                &[
                    Point2 { x: x1, y: y1 },
                    Point2 { x: x2, y: y2 },
                ],
                1.0,
                Color::from_rgb(128, 128, 128),
            )?;
            canvas.draw(&line, DrawParam::default());
        }

        // nodes
        for &(x, y, ref label, is_notable) in &self.node_positions {
            let color = if is_notable { Color::YELLOW } else { Color::BLUE };
            let circle = Mesh::new_circle(
                ctx,
                DrawMode::fill(),
                Point2 { x, y },
                if is_notable { 6.0 } else { 3.0 },
                0.1,
                color,
            )?;
            canvas.draw(&circle, DrawParam::default());

            if is_notable {
                let text = Text::new(label.clone());
                canvas.draw(&text, DrawParam::default().dest([x + 10.0, y]));
            }
        }

        canvas.finish(ctx)
    }
}

// ----------------------------------------------------
// Main
// ----------------------------------------------------
fn main() -> GameResult {
    let data = load_tree("POE2_TREE.json");
    let (ctx, event_loop) = ContextBuilder::new("passive_tree", "ggez")
        .window_setup(conf::WindowSetup::default().title("Path of Exile Passive Tree"))
        .window_mode(conf::WindowMode::default().dimensions(1920.0, 1080.0))
        .build()?;

    let vis = TreeVisualization::new(data);

    event::run(ctx, event_loop, vis)
}
