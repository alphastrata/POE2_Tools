//$ crates/poe_tree/src/edges.rs
use std::hash::{Hash, Hasher};

use super::type_wrappings::NodeId;
use super::PassiveTree;

#[derive(Debug, Clone)]
pub struct Edge {
    // id: EdgeId,
    pub start: NodeId,
    pub end: NodeId,
}
impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        (self.start == other.start && self.end == other.end)
            || (self.start == other.end && self.end == other.start)
    }
}

impl Eq for Edge {}

impl Hash for Edge {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let min = std::cmp::min(self.start, self.end);
        let max = std::cmp::max(self.start, self.end);
        min.hash(state);
        max.hash(state);
    }
}

impl Edge {
    pub fn new(from: NodeId, to: NodeId, tree: &super::PassiveTree) -> Self {
        // Determine the closer node to 0,0
        let from_node = &tree.nodes[&from];
        let to_node = &tree.nodes[&to];
        let dist_from = from_node.distance_to_origin();
        let dist_to = to_node.distance_to_origin();

        if dist_from <= dist_to {
            Edge {
                start: from,
                end: to,
            }
        } else {
            Edge {
                start: to,
                end: from,
            }
        }
    }
}

impl PassiveTree {
    pub fn get_edges(&self) -> Vec<(NodeId, NodeId)> {
        self.edges
            .iter()
            .map(|edge| (edge.start, edge.end))
            .collect()
    }
}
