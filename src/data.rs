// src/data.rs
use std::collections::HashMap;

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
    // Use usize for IDs
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
    pub active: bool, // set when clicked
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

