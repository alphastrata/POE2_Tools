
use poo_tree::type_wrappings::NodeId;

use super::*;
// IO
impl TreeVis<'_> {
    pub fn select_node(&mut self, node_id: usize) {
        if let Some(character) = &mut self.current_character {
            if !character.activated_node_ids.contains(&node_id) {
                log::info!("Selecting node: {}", node_id);
                self.target_node_id = node_id;
                self.process_path_to_active_node(node_id, true); // Activate the path
            }
        }
    }
    pub fn hover_node(&mut self, node_id: usize) {
        if let Some(node) = self.passive_tree.nodes.get(&node_id) {
            if !node.active {
                log::info!("Hovering over node: {}", node_id);

                // Highlight the shortest path to an active node (do not activate)
                if let Some(shortest_path) = self.process_path_to_active_node(node_id, false) {
                    // log::debug!("Highlighting path: {:?}", shortest_path);
                    self.highlighted_path = shortest_path;
                }
            }
        }
    }

    /// Processes the shortest path from a given node to the nearest active node.
    /// If `activate` is true, nodes and edges along the path will be activated.
    pub fn process_path_to_active_node(
        &mut self,
        node_id: usize,
        activate: bool,
    ) -> Option<Vec<usize>> {
        if let Some((shortest_path, active_node_id)) =
            self.find_shortest_path_to_active_node(node_id)
        {
            log::debug!(
                "Shortest path from {} to {}: {:?}",
                node_id,
                active_node_id,
                shortest_path
            );

            if activate {
                // Activate nodes along the path
                shortest_path.iter().for_each(|&path_node_id| {
                    if let Some(node) = self.passive_tree.nodes.get_mut(&path_node_id) {
                        if !node.active {
                            log::debug!("Activating node: {}", path_node_id);
                            node.active = true;
                            self.active_nodes.insert(path_node_id);
                        }
                    }
                });

                // Update active edges
                self.update_active_edges(shortest_path.clone());
            }

            return Some(shortest_path);
        } else {
            log::warn!("No active node found to connect to {}", node_id);
        }

        None
    }

    /// Finds the shortest path from the given node to the nearest active node.
    pub fn find_shortest_path_to_active_node(
        &self,
        target_node: usize,
    ) -> Option<(Vec<usize>, usize)> {
        self.passive_tree
            .nodes
            .iter()
            .filter(|(_, node)| node.active)
            .map(|(&active_node_id, _)| {
                let path = self.passive_tree.find_path(target_node, active_node_id);
                (path, active_node_id)
            })
            .filter(|(path, _)| !path.is_empty())
            .min_by_key(|(path, _)| path.len())
    }

    /// Updates the active edges based on the given path.
    pub fn update_active_edges(&mut self, path: Vec<usize>) {
        for window in path.windows(2) {
            if let [start, end] = window {
                self.active_edges.insert((*start, *end));
                self.active_edges.insert((*end, *start));
                log::debug!("Edge activated: ({}, {})", start, end);
            }
        }
    }

    pub(crate) const HOVER_RADIUS: f32 = 100.0;
    pub fn get_hovered_node(&self, ctx: &egui::Context) -> Option<usize> {
        let mouse_pos = ctx.input(|input| input.pointer.hover_pos())?;

        self.passive_tree.nodes.iter().find_map(|(&node_id, node)| {
            let screen_x = self.world_to_screen_x(node.wx);
            let screen_y = self.world_to_screen_y(node.wy);

            let distance =
                ((mouse_pos.x - screen_x).powi(2) + (mouse_pos.y - screen_y).powi(2)).sqrt();
            if distance < Self::HOVER_RADIUS {
                Some(node_id)
            } else {
                None
            }
        })
    }

    pub fn get_target_node(&self) -> Option<usize> {
        // Logic to determine if a target node has been selected
        Some(self.target_node_id)
    }
    pub fn handle_mouse(&mut self, ctx: &egui::Context) {
        ctx.input(|input| {
            if let Some(mouse_pos) = input.pointer.hover_pos() {
                // Handle zoom and drag interactions
                if input.raw_scroll_delta.y != 0.0 {
                    let raw_scroll = input.raw_scroll_delta.y;
                    self.adjust_zoom(raw_scroll, mouse_pos);
                }
                if input.pointer.primary_down() {
                    let mouse_delta = input.pointer.delta();
                    self.translate_camera(-mouse_delta.x, -mouse_delta.y);
                }

                // Update  node
                self.update_hovered_node(mouse_pos);
            }
        });
    }
    /// Update the hovered node based on mouse position.
    pub fn update_hovered_node(&mut self, mouse_pos: egui::Pos2) {
        let mut closest_node = None;
        let mut closest_distance = f32::MAX;

        self.passive_tree.nodes.values().for_each(|node| {
            let sx = self.world_to_screen_x(node.wx);
            let sy = self.world_to_screen_y(node.wy);

            let distance = ((mouse_pos.x - sx).powi(2) + (mouse_pos.y - sy).powi(2)).sqrt();

            //todo: from config
            if distance < 15.0 && distance < closest_distance {
                closest_node = Some(node.node_id);
                closest_distance = distance;
            }
        });

        self.hovered_node = closest_node;
    }

    pub fn click_node(&mut self, node_id: NodeId) {
        if let Some(node) = self.passive_tree.nodes.get_mut(&node_id) {
            node.active = !node.active;
            if node.active {
                if !self.highlighted_path.contains(&node_id) {
                    self.highlighted_path.push(node_id);
                }
            } else {
                self.highlighted_path.retain(|&id| id != node_id);
            }
        }
        self.requires_activation_check = true; // set flag
    }
    pub fn clear_active_nodes(&mut self) {
        for node in self.passive_tree.nodes.values_mut() {
            node.active = false;
        }
        self.highlighted_path.clear();
        self.active_edges.clear();
        log::info!("Cleared all active nodes and paths.");
    }
}
