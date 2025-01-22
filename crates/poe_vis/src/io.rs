//!$ crates/poe_vis/src/io.rs
use poe_tree::type_wrappings::NodeId;

use super::*;
// IO
impl TreeVis<'_> {
    pub fn select_node(&mut self, node_id: u32) {
        self.target_node_id = node_id;
        self.process_path_to_active_node(node_id, true); // Activate the path
    }
    pub fn hover_node(&mut self, node_id: u32) {
        if let Some(node) = self.passive_tree.nodes.get(&node_id) {
            if !node.active {
                log::trace!("Hovering over node: {}", node_id);

                // Highlight the shortest path to an active node (do not activate)
                if let Some(shortest_path) = self.process_path_to_active_node(node_id, false) {
                    // log::debug!("Highlighting path: {:?}", shortest_path);
                    self.highlighted_path = shortest_path;
                }
            }
        }
    }

    /// Processes the shortest path from a given node to the nearest active node.
    /// If `activate` is true, nodes and edges along the path will be activated,
    /// activating the edges resulting in a highlighted path and a character update
    pub fn process_path_to_active_node(
        &mut self,
        node_id: u32,
        activate: bool,
    ) -> Option<Vec<u32>> {
        if let Some((shortest_path, active_node_id)) =
            self.find_shortest_path_to_active_node(node_id)
        {
            log::trace!(
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
            log::trace!("No active node found to connect to {}", node_id);
        }

        None
    }

    /// Finds the shortest path from the given node to the nearest active node.
    pub fn find_shortest_path_to_active_node(&self, target_node: u32) -> Option<(Vec<u32>, u32)> {
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
    pub fn update_active_edges(&mut self, path: Vec<u32>) {
        for window in path.windows(2) {
            if let [start, end] = window {
                self.active_edges.insert((*start, *end));
                self.active_edges.insert((*end, *start));
                log::debug!("Edge activated: ({}, {})", start, end);
            }
        }
    }

    pub(crate) const HOVER_RADIUS: f32 = 100.0;
    pub fn get_hovered_node(&self, ctx: &egui::Context) -> Option<u32> {
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

    pub fn get_target_node(&self) -> Option<u32> {
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
            log::debug!("Flipping node {} to active= {}", node_id, node.active);
            if node.active {
                if !self.highlighted_path.contains(&node_id) {
                    self.highlighted_path.push(node_id);
                }
            } else {
                // When we deactivate a node we potentially need to update three of our out-of-bounds tracking lists.
                // tedious cleanup indeed.
                self.highlighted_path.retain(|&id| id != node_id);
                self.active_edges
                    .retain(|(a, b)| a != &node_id || b != &node_id);
                self.active_nodes.retain(|nid| nid != &node_id);
            }
        }

        // A little spammy but we always rebuild our 'paths' to ensure they're valid post editing the connected nodes we have etc.
        self.requires_activation_check = true;
    }

    /// Clears all active nodes and edges (except the self.starting_id)
    pub fn clear_active_nodes_and_edges(&mut self) {
        self.passive_tree
            .nodes
            .values_mut()
            .filter(|n| n.node_id == self.start_node_id)
            .for_each(|node| {
                node.active = false;
            });
        self.highlighted_path.clear();
        self.active_edges.clear();
        log::info!("Cleared all active nodes and paths.");
    }
}
impl TreeVis<'_> {
    pub fn handle_keyboard(&mut self, ctx: &egui::Context) {
        ctx.input(|input| {
            // Trigger fuzzy search popup on `/`
            if input.key_pressed(egui::Key::Slash) {
                self.open_fuzzy_search();
            }

            // Exit application on `Escape`
            if input.key_pressed(egui::Key::Escape) {
                std::process::exit(0);
            }
        });
    }
}

// pub mod search {
// use super::TreeVis;
//}
// Adjusted code without `.hint_text` since `Response` doesn't have that method
impl TreeVis<'_> {
    pub fn show_fuzzy_search_popup(&mut self, ctx: &egui::Context) {
        if let Some(mouse_pos) = ctx.input(|i| i.pointer.hover_pos()) {
            let popup_id = egui::Id::new("fuzzy_search_popup");

            egui::Window::new("🔍 Fuzzy Search")
                .id(popup_id)
                .collapsible(false)
                .resizable(false)
                .fixed_pos(mouse_pos) // Position the popup at the mouse location
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]) // Center the popup
                .show(ctx, |ui| {
                    ui.label("🔍 Search");

                    // Text input for the search query
                    let response = ui.text_edit_singleline(&mut self.search_query);
                    if response.changed() {
                        if !self.search_query.is_empty() {
                            self.search_results =
                                self.passive_tree.fuzzy_search_nodes(&self.search_query);
                            log::debug!("Search query updated: {}", self.search_query);
                            log::debug!("Search results: {:?}", self.search_results);
                        } else {
                            self.search_results.clear();
                            log::debug!("Search query cleared, search results reset.");
                        }
                    }

                    // Request focus for the text input field
                    response.request_focus();

                    // Display search results
                    if !self.search_query.is_empty() {
                        ui.separator();
                        ui.label("Search results:");
                        for &node_id in &self.search_results {
                            if let Some(node) = self.passive_tree.nodes.get(&node_id) {
                                ui.horizontal(|ui| {
                                    if ui.button(&node.name).clicked() {
                                        self.go_to_node(node_id);
                                        self.close_fuzzy_search();
                                        log::debug!(
                                            "Navigated to node: {} ({})",
                                            node.name,
                                            node_id
                                        );
                                    }
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui.button("📋").clicked() {
                                                ui.output_mut(|o| {
                                                    o.copied_text = node_id.to_string()
                                                });
                                                log::info!(
                                                    "Copied Node ID {} to clipboard",
                                                    node_id
                                                );
                                            }
                                        },
                                    );
                                });
                            }
                        }
                    }

                    // Close the popup on Enter key
                    ctx.input(|input| {
                        if input.key_pressed(egui::Key::Enter) {
                            ui.close_menu(); // Close the popup
                        }
                    });
                });
        }
    }
}
