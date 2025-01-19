//$ crates/poe_tree/src/data/poe_tree/nodes.rs
use crate::config::{parse_color, UserConfig};

use super::stats::Stat;
use super::type_wrappings::{GroupId, NodeId};
use super::PassiveTree;

#[derive(Debug, Clone, Default)]
pub struct PoeNode {
    pub node_id: NodeId,
    pub skill_id: String,
    pub parent: GroupId,
    pub radius: u8,
    pub position: usize,
    pub name: String,
    pub is_notable: bool,
    pub stats: Vec<Stat>,
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

    pub fn base_color(&self, config: &UserConfig) -> egui::Color32 {
        let name = self.name.to_lowercase();
        if Self::INTELLIGENCE_KEYWORDS
            .iter()
            .any(|&kw| name.contains(kw))
        {
            return parse_color(config.colors.get("intelligence").unwrap());
        }
        if Self::DEXTERITY_KEYWORDS.iter().any(|&kw| name.contains(kw)) {
            return parse_color(config.colors.get("dexterity").unwrap());
        }
        if Self::STRENGTH_KEYWORDS.iter().any(|&kw| name.contains(kw)) {
            return parse_color(config.colors.get("strength").unwrap());
        }

        parse_color(config.colors.get("all_nodes").unwrap())
    }
}
