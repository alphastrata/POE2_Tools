//$ src/data/poe_tree/nodes.rs
use super::stats::{Operand, Stat};
use super::type_wrappings::{EdgeId, GroupId, NodeId};
use super::{consts::*, PassiveTree};

#[derive(Debug, Clone, Default)]
pub struct PoeNode {
    // Removed lifetime parameter
    pub node_id: NodeId,
    pub skill_id: String,
    pub parent: GroupId,
    pub radius: u8,
    pub position: usize,
    pub name: String,
    pub is_notable: bool,
    pub stats: Vec<Stat>, // Now we own the stats
    pub wx: f64,
    pub wy: f64,
    pub active: bool,
}

impl PoeNode {
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
