//!$ crates/poe_tree/src/nodes.rs
use super::type_wrappings::{GroupId, NodeId};
use super::PassiveTree;
use crate::skills::PassiveSkill;

#[derive(Debug, Clone, Default)]
pub struct PoeNode {
    pub node_id: NodeId,
    pub skill_id: String,
    pub parent: GroupId,
    pub radius: u8,
    pub position: NodeId, // It's not actually, but as it'll be < the precision we use for a NodeId
    pub name: String,
    pub is_notable: bool,
    pub wx: f32,
    pub wy: f32,
    pub active: bool,
}

impl PoeNode {
    pub fn contains_stat_with_keyword(&self, tree: &PassiveTree, keyword: &str) -> bool {
        self.as_passive_skill(tree)
            .stats()
            .iter()
            .any(|stat| stat.as_str().contains(keyword))
    }
    pub fn contains_keyword(&self, keyword: &str) -> bool {
        self.skill_id.contains(keyword) || self.name.contains(keyword)
    }
    pub fn num_points_required_from(&self, other: NodeId, tree: &PassiveTree) -> usize {
        tree.bfs(self.node_id, other).len()
    }

    pub fn distance_to(&self, other: &Self) -> f32 {
        ((self.wx - other.wx).powi(2) + (self.wy - other.wy).powi(2)).sqrt()
    }

    pub fn distance_to_origin(&self) -> f32 {
        (self.wx.powi(2) + self.wy.powi(2)).sqrt()
    }

    pub fn path_to_target(&self, target: NodeId, tree: &PassiveTree) -> Vec<NodeId> {
        tree.find_shortest_path(self.node_id, target)
    }

    pub const INTELLIGENCE_KEYWORDS: [&'static str; 6] = [
        "intelligence",
        "energy shield",
        "lightning",
        "spell damage",
        "critical strike chance",
        "critical damage",
    ];
    pub const DEXTERITY_KEYWORDS: [&'static str; 6] = [
        "evasion",
        "dexterity",
        "movement speed",
        "attack speed",
        "skill speed",
        "spell speed",
    ];
    pub const STRENGTH_KEYWORDS: [&'static str; 5] = [
        "attack damage",
        "melee damage",
        "physical damage",
        "maximum life",
        "life on kill",
    ];

    pub fn as_passive_skill<'t>(&self, tree: &'t PassiveTree) -> &'t PassiveSkill {
        tree.passive_for_node(self)
    }
}
