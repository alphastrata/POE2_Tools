//$ src\data\poe_tree\nodes.rs
use super::coordinates::Group;
use super::skills::PassiveSkill;
use super::stats::{Operand, Stat};
use super::type_wrappings::{EdgeId, GroupId, NodeId};
use super::{consts::*, PassiveTree};
use serde_json::Value;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::{collections::VecDeque, fs};

#[derive(Debug, Clone, Default)]
pub struct PoeNode<'stat> {
    pub node_id: NodeId,
    pub skill_id: Option<String>,
    pub parent: GroupId,
    pub radius: u8,
    pub position: usize,
    pub name: String,
    pub is_notable: bool,
    pub stats: &'stat [Stat],
    pub wx: f64,
    pub wy: f64,
    pub active: bool,
}

impl<'stat> PoeNode<'stat> {
    pub fn distance_to(&self, other: &Self) -> f64 {
        ((self.wx - other.wx).powi(2) + (self.wy - other.wy).powi(2)).sqrt()
    }

    pub fn distance_to_origin(&self) -> f64 {
        (self.wx.powi(2) + self.wy.powi(2)).sqrt()
    }

    pub fn path_to_target(&self, target: NodeId, tree: &PassiveTree) -> Vec<NodeId> {
        tree.find_shortest_path(self.node_id, target)
    }
}
//$ src\data\poe_tree\nodes.rs
use super::coordinates::Group;
use super::skills::PassiveSkill;
use super::stats::{Operand, Stat};
use super::{consts::*, PassiveTree};
use serde_json::Value;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::{collections::VecDeque, fs};
pub type GroupId = usize;
pub type NodeId = usize;

#[derive(Debug, Clone, Default)]
pub struct PoeNode<'stat> {
    pub node_id: NodeId,
    pub skill_id: Option<String>,
    pub parent: GroupId,
    pub radius: u8,
    pub position: usize,
    pub name: String,
    pub is_notable: bool,
    pub stats: &'stat [Stat],
    pub wx: f64,
    pub wy: f64,
    pub active: bool,
}

impl<'stat> PoeNode<'stat> {
    pub fn distance_to(&self, other: &Self) -> f64 {
        ((self.wx - other.wx).powi(2) + (self.wy - other.wy).powi(2)).sqrt()
    }

    pub fn distance_to_origin(&self) -> f64 {
        (self.wx.powi(2) + self.wy.powi(2)).sqrt()
    }

    pub fn path_to_target(&self, target: NodeId, tree: &PassiveTree) -> Vec<NodeId> {
        tree.find_shortest_path(self.node_id, target)
    }
}
