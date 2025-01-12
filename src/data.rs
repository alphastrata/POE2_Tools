// src/data.rs

use egui::accesskit::Tree;
use serde_json::Value;
use std::{collections::HashMap, fs};

pub const ORBIT_RADII: [f64; 8] = [0.0, 82.0, 162.0, 335.0, 493.0, 662.0, 812.0, 972.0];
pub const ORBIT_SLOTS: [usize; 8] = [1, 6, 16, 16, 40, 60, 60, 60];

#[derive(Debug, Clone)]
pub struct Group {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone)]
pub struct PassiveSkill {
    pub name: Option<String>,
    pub is_notable: bool,
    pub stats: Vec<(String, f64)>, // no more HashMap
}

#[derive(Debug, Clone)]
pub struct Node {
    pub skill_id: Option<String>,
    pub parent: usize,
    pub radius: usize,
    pub position: usize,
    pub connections: Vec<usize>,
    // Derived data
    pub name: String,
    pub is_notable: bool,
    pub stats: Vec<(String, f64)>,
    pub wx: f64,
    pub wy: f64,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct PassiveTree {
    pub groups: HashMap<usize, Group>,
    pub nodes: HashMap<usize, Node>,
}

#[derive(Debug, Clone)]
pub struct TreeData {
    pub passive_tree: PassiveTree,
    pub passive_skills: HashMap<String, PassiveSkill>,
}

impl TreeData {
    pub fn compute_positions_and_stats(&mut self) {
        for (_, node) in self.passive_tree.nodes.iter_mut() {
            // 1) group pos
            if let Some(group) = self.passive_tree.groups.get(&node.parent) {
                // radius
                let r = ORBIT_RADII.get(node.radius).copied().unwrap_or(0.0);
                // how many slots on this orbit
                let slots = ORBIT_SLOTS.get(node.radius).copied().unwrap_or(1) as f64;
                // convert node.position into an angle
                let angle = node.position as f64 * (2.0 * std::f64::consts::PI / slots);

                node.wx = group.x + r * angle.cos();
                node.wy = group.y + r * angle.sin();
            }

            // 2) fill name, is_notable, stats from skill table
            if let Some(skill_id) = &node.skill_id {
                if let Some(skill) = self.passive_skills.get(skill_id) {
                    node.name = skill.name.clone().unwrap_or_default();
                    node.is_notable = skill.is_notable;
                    node.stats = skill.stats.clone();
                }
            }
        }
    }

    pub fn load_tree(path: &str) -> Self {
        let data = fs::read_to_string(path).expect("Failed to read JSON");
        let json: Value = serde_json::from_str(&data).expect("Invalid JSON");

        // parse groups
        let mut groups = HashMap::new();
        if let Some(obj) = json["passive_tree"]["groups"].as_object() {
            for (gid, gval) in obj {
                let gx = gval["x"].as_f64().unwrap_or(0.0);
                let gy = gval["y"].as_f64().unwrap_or(0.0);
                groups.insert(
                    gid.parse::<usize>().unwrap_or_default(),
                    Group { x: gx, y: gy },
                );
            }
        }

        // parse nodes
        let mut nodes = HashMap::new();
        if let Some(obj) = json["passive_tree"]["nodes"].as_object() {
            for (nid, nval) in obj {
                let skill_id = nval["skill_id"].as_str().map(|s| s.to_string());
                let parent = nval["parent"].as_u64().unwrap_or(0) as usize;
                let radius = nval["radius"].as_u64().unwrap_or(0) as usize;
                let position = nval["position"].as_u64().unwrap_or(0) as usize;

                let mut connections = Vec::new();
                if let Some(conn_arr) = nval["connections"].as_array() {
                    for c in conn_arr {
                        if let Some(cid) = c["id"].as_u64() {
                            connections.push(cid as usize);
                        }
                    }
                }

                nodes.insert(
                    nid.parse::<usize>().unwrap_or_default(),
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
                        active: false,
                    },
                );
            }
        }

        // parse passive_skills
        let mut passive_skills = HashMap::new();
        if let Some(skills_obj) = json["passive_skills"].as_object() {
            for (skill_id, sval) in skills_obj {
                let name = sval["name"].as_str().map(|s| s.to_string());
                let is_notable = sval["is_notable"].as_bool().unwrap_or(false);

                // Collect stats into Vec<(String, f64)>
                let mut stats_vec = Vec::new();
                if let Some(st) = sval["stats"].as_object() {
                    for (k, v) in st {
                        if let Some(num) = v.as_f64() {
                            stats_vec.push((k.clone(), num));
                        }
                    }
                }

                passive_skills.insert(
                    skill_id.clone(),
                    PassiveSkill {
                        name,
                        is_notable,
                        stats: stats_vec,
                    },
                );
            }
        }

        Self {
            passive_tree: PassiveTree { groups, nodes },
            passive_skills,
        }
    }
}
