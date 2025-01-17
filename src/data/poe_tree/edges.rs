//$ src/data/poe_tree/edges.rs
use super::type_wrappings::NodeId;
use super::{consts::*, PassiveTree};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Edge {
    // id: EdgeId,
    pub from: NodeId,
    pub to: NodeId,
}

impl Edge {
    pub fn new(from: NodeId, to: NodeId, tree: &super::PassiveTree) -> Self {
        // Determine the closer node to 0,0
        let from_node = &tree.nodes[&from];
        let to_node = &tree.nodes[&to];
        let dist_from = from_node.distance_to_origin();
        let dist_to = to_node.distance_to_origin();

        if dist_from <= dist_to {
            Edge { from, to }
        } else {
            Edge { from: to, to: from }
        }
    }
}

impl PassiveTree {
    pub fn get_edges(&self) -> Vec<(NodeId, NodeId)> {
        self.edges.iter().map(|edge| (edge.from, edge.to)).collect()
    }

    pub(crate) fn compute_positions_and_stats(&mut self) {
        for (_, node) in self.nodes.iter_mut() {
            // Compute world positions (wx, wy) using the group and radius information
            if let Some(group) = self.groups.get(&node.parent) {
                let radius = ORBIT_RADII
                    .get(node.radius as usize)
                    .copied()
                    .unwrap_or(0.0);

                let slots = ORBIT_SLOTS.get(node.radius as usize).copied().unwrap_or(1) as f64;
                let angle = node.position as f64 * (2.0 * std::f64::consts::PI / slots);

                node.wx = group.x + radius * angle.cos();
                node.wy = group.y + radius * angle.sin();
            }

            // Populate derived data: name, is_notable, stats
            if let Some(skill) = self.passive_skills.get(&node.skill_id) {
                node.name = skill.name.clone().unwrap_or_default();
                node.is_notable = skill.is_notable;
                node.stats = skill.stats.clone();
            }
        }
    }
}
