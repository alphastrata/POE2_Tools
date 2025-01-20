//$ crates/poe_vis/src/background_services.rs
use std::{
    collections::HashSet,
    ops::ControlFlow,
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};

pub(crate) static ACTIVE_NODE_COUNT: AtomicUsize = AtomicUsize::new(0);
use poe_tree::type_wrappings::NodeId;

use super::*;

impl TreeVis<'_> {
    pub fn set_start_node(&mut self, ctx: &egui::Context) {
        if let Some((&closest_id, closest_node)) =
            self.passive_tree.nodes.iter().min_by(|(_, a), (_, b)| {
                let dist_a = (a.wx.powi(2) + a.wy.powi(2)).sqrt();
                let dist_b = (b.wx.powi(2) + b.wy.powi(2)).sqrt();
                dist_a
                    .partial_cmp(&dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        {
            self.start_node_id = closest_id;
            log::info!("Start node set to ID: {}", closest_id);

            // Draw a small white triangle at the start node
            let painter = ctx.layer_painter(egui::LayerId::background());
            let sx = self.world_to_screen_x(closest_node.wx);
            let sy = self.world_to_screen_y(closest_node.wy);

            painter.add(egui::Shape::convex_polygon(
                vec![
                    egui::pos2(sx, sy - 5.0),       // Top point of the triangle
                    egui::pos2(sx - 4.0, sy + 3.0), // Bottom-left point
                    egui::pos2(sx + 4.0, sy + 3.0), // Bottom-right point
                ],
                egui::Color32::WHITE,
                egui::Stroke::NONE,
            ));
        }
    }

    pub fn check_and_activate_nodes(&mut self) {
        let active_nodes: HashSet<usize> = self
            .passive_tree
            .nodes
            .iter()
            .filter(|(_, node)| node.active)
            .map(|(id, _)| *id)
            .collect();

        let current_active_count = active_nodes.len();
        let previous_active_count = ACTIVE_NODE_COUNT.load(Ordering::Relaxed);

        if current_active_count <= previous_active_count {
            return;
        }

        ACTIVE_NODE_COUNT.store(current_active_count, Ordering::Relaxed);

        if active_nodes.len() < 2 {
            log::debug!("Not enough active nodes for pathfinding.");
            return;
        }

        let mut visited_nodes: HashSet<NodeId> = HashSet::new();
        let mut updated_nodes = false;

        active_nodes.iter().enumerate().try_for_each(|(i, &start)| {
            active_nodes.iter().skip(i + 1).try_for_each(|&end| {
                if visited_nodes.contains(&start) && visited_nodes.contains(&end) {
                    return ControlFlow::Continue::<()>(());
                }

                log::debug!("Attempting to find path between {} and {}", start, end);

                if !self.passive_tree.edges.iter().any(|edge| {
                    (edge.start == start && edge.end == end)
                        || (edge.start == end && edge.end == start)
                }) {
                    let start_time = Instant::now();
                    let path = self.passive_tree.find_path(start, end);

                    if !path.is_empty() {
                        log::debug!("Path found: {:?} between {} and {}", path, start, end);

                        path.iter().for_each(|&node_id| {
                            if let Some(node) = self.passive_tree.nodes.get_mut(&node_id) {
                                if !node.active {
                                    node.active = true;
                                    updated_nodes = true;
                                }
                                visited_nodes.insert(node_id);
                            }
                        });

                        let duration = start_time.elapsed();
                        log::debug!("Path activated in {:?}", duration);
                    } else {
                        log::debug!("No path found between {} and {}", start, end);
                    }
                }

                ControlFlow::Continue::<()>(())
            })
        });

        if updated_nodes {
            log::debug!("Nodes activated along paths.");
            self.active_nodes = active_nodes;
        } else {
            log::debug!("No new nodes activated.");
        }
    }

    pub fn check_and_activate_edges(&mut self) {
        let mut visited_edges: HashSet<(NodeId, NodeId)> = HashSet::new();
        let active_nodes: Vec<_> = self
            .passive_tree
            .nodes
            .iter()
            .filter(|(_, node)| node.active)
            .map(|(id, _)| *id)
            .collect();

        // Don't recompute paths and edges unless we've increased the number of nodes
        let current_active_count = active_nodes.len();
        let previous_active_count = ACTIVE_NODE_COUNT.load(Ordering::Relaxed);
        if current_active_count <= previous_active_count {
            return;
        }

        active_nodes.iter().enumerate().try_for_each(|(i, &start)| {
            active_nodes.iter().skip(i + 1).try_for_each(|&end| {
                if visited_edges.contains(&(start, end)) || visited_edges.contains(&(end, start)) {
                    return ControlFlow::Continue::<()>(());
                }

                if self.passive_tree.edges.iter().any(|edge| {
                    (edge.start == start && edge.end == end)
                        || (edge.start == end && edge.end == start)
                }) {
                    log::debug!("Edge found and activated between {} and {}", start, end);

                    visited_edges.insert((start, end));
                }

                ControlFlow::Continue::<()>(())
            })
        });
        self.active_edges = visited_edges;

        log::debug!(
            "Edge activation completed. Active edges: {:?}",
            self.active_edges
        );
    }
}
