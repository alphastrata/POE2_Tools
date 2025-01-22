//!$ crates/poe_vis/src/background_services.rs
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
        let active_nodes: HashSet<u32> = self
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
        log::debug!(
            "Nodes in the 'visited' we just looked at: [ {:#?} ]",
            &visited_edges
        );

        self.active_edges = visited_edges;

        log::debug!(
            "Edge activation completed. Active edges: {:?}",
            self.active_edges
        );
        log::debug!("Active edges:     {:?}", self.active_edges.len());
        log::debug!("Active nodes:     {:?}", self.active_nodes.len());
        log::debug!("highlighted path: {:?}", self.highlighted_path.len());
    }
}

impl TreeVis<'_> {
    pub const DEFAULT_CHARACTER_SAVE_PATH: &'static str = "./data/character.toml";

    pub fn character(&self) -> &Character {
        &self.user_config.character
    }
    //TODO: rate-limit
    pub(crate) fn update_character(&mut self) {
        if self.active_nodes != self.character().activated_node_ids {
            self.active_nodes
                .insert(self.user_config.character.starting_node);
            self.highlighted_path
                .push(self.user_config.character.starting_node);
            self.active_nodes.remove(&0); // remove the default placeholder JIC

            self.user_config.character.activated_node_ids = self.active_nodes.clone();

            // NOTE: auto-save on changes is disabled.
            // self.save_character();
        } else {
            log::trace!("Character data unchanged, no update required");
        }
    }

    pub fn save_character(&mut self) {
        if let Err(e) = self
            .character()
            .save_to_toml(Self::DEFAULT_CHARACTER_SAVE_PATH)
        {
            log::error!("Unable to save Character updates, Error: {e}");
        };
    }

    /// Loads a character, sets the uploaded character's path to ours on Self.
    /// sets the checks flag so we rehighlight paths etc.
    pub fn load_character<P: AsRef<std::path::Path>>(&mut self, path: P) {
        let p = path.as_ref();

        match Character::load_from_toml(p) {
            Some(c) => {
                log::debug!("Character before loading: {}", self.character());
                log::debug!(
                    "TreeVis before loading:\n starting_id:{}, active_nodes{}",
                    self.start_node_id,
                    self.active_nodes.len()
                );

                self.start_node_id = c.starting_node;
                self.active_nodes = c.activated_node_ids.clone();
                let loaded_active_nodes = c
                    .activated_node_ids
                    .iter()
                    .filter_map(|v| match self.passive_tree.nodes.get_mut(v) {
                        Some(m_node) => {
                            m_node.active = true;
                            Some(())
                        }
                        None => None,
                    })
                    .count();

                assert_eq!(loaded_active_nodes, self.active_nodes.len());
                self.user_config.character = c;
                log::debug!("Character after loading: {}", self.character());
                log::debug!(
                    "TreeVis after loading:\n starting_id:{}, active_nodes{}",
                    self.start_node_id,
                    loaded_active_nodes
                );
                self.requires_activation_check = true;
            }
            None => {
                log::error!("Unable to load Character from path :{}", p.display());
            }
        }
    }
}

impl TreeVis<'_> {
    /*
    Initial State:

        start_node_id = 1
        active_nodes = {1, 2, 3, 4, 5, 7, 8}
        active_edges = { (1, 2), (2, 3), (3, 4), (4, 5), (5, 7), (7, 8) }

    Node Removal:

        User removes 5.

    After Repair:

        Nodes 7 and 8 are unreachable.
        active_nodes = {1, 2, 3, 4}
        active_edges = { (1, 2), (2, 3), (3, 4) }
        highlighted_path is recalculated or cleared.
    */
    pub fn repair_broken_paths(&mut self) {
        // Debug initial state
        // dbg!(
        //     &self.active_edges,
        //     &self.active_nodes,
        //     &self.highlighted_path
        // );

        // // Record the initial state for delta comparison
        // let initial_active_nodes = self.active_nodes.clone();
        // let initial_active_edges = self.active_edges.clone();

        // Step 1: Find all reachable nodes from `self.start_node_id`
        let mut reachable_nodes = HashSet::new();
        let mut repaired_edges = HashSet::new();

        self.active_nodes.iter().for_each(|&node_id| {
            let path = self.passive_tree.bfs(self.start_node_id, node_id);

            reachable_nodes.insert(node_id);

            // Add edges from the valid path
            path.windows(2).for_each(|window| {
                if let [start, end] = window {
                    repaired_edges.insert((*start, *end));
                }
            });
        });

        // Step 2: Remove unreachable nodes and associated edges
        self.active_nodes
            .difference(&reachable_nodes)
            .cloned()
            .for_each(|unreachable| {
                log::warn!("Removing unreachable node: {}", unreachable);

                // Remove edges involving the unreachable node
                self.active_edges
                    .retain(|&(start, end)| start != unreachable && end != unreachable);
            });

        // Step 3: Update active nodes and edges
        self.active_nodes = reachable_nodes;
        self.active_edges = repaired_edges;

        // Step 4: Recalculate `highlighted_path`
        self.highlighted_path = self
            .passive_tree
            .bfs(self.start_node_id, self.target_node_id);

        // Debug deltas directly without undeclared bindings
        // dbg!({
        //     let delta_nodes: HashSet<_> = initial_active_nodes
        //         .difference(&self.active_nodes)
        //         .cloned()
        //         .collect();
        //     let delta_edges: HashSet<_> = initial_active_edges
        //         .difference(&self.active_edges)
        //         .cloned()
        //         .collect();
        //     (delta_nodes, delta_edges)
        // });

        // // Debug final state
        // dbg!(
        //     &self.active_edges,
        //     &self.active_nodes,
        //     &self.highlighted_path
        // );
    }
}
