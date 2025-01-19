use std::default::Default;


use poo_tree::config::parse_color;

use super::*;
// drawing{
pub mod rhs_menu {
    use super::TreeVis;

    impl TreeVis<'_> {
        pub fn draw_rhs_menu(&mut self, ctx: &egui::Context) {
            let rhs_rect = egui::SidePanel::right("rhs_menu")
                .show(ctx, |ui| {
                    ui.heading("Menu");

                    // Top Buttons Section
                    self.draw_top_buttons(ui);

                    // Search Functionality
                    self.search(ui);

                    // Start and Target Node Configuration
                    self.pathing(ui);
                })
                .response
                .rect; // Capture the rectangle of the RHS menu

            // Log the RHS menu dimensions
            log::debug!(
                "RHS menu rect: x = {:.2}, y = {:.2}, width = {:.2}, height = {:.2}",
                rhs_rect.min.x,
                rhs_rect.min.y,
                rhs_rect.width(),
                rhs_rect.height(),
            );

            // Get and log the mouse position
            if let Some(mouse_pos) = ctx.input(|input| input.pointer.hover_pos()) {
                log::debug!(
                    "Mouse position: x = {:.2}, y = {:.2}",
                    mouse_pos.x,
                    mouse_pos.y,
                );

                // Overlay debugging information on the screen
                let painter = ctx.layer_painter(egui::LayerId::debug());
                painter.text(
                    egui::pos2(mouse_pos.x, mouse_pos.y),
                    egui::Align2::LEFT_BOTTOM,
                    format!(
                        "Mouse: x = {:.2}, y = {:.2}\nRHS: x = {:.2}-{:.2}, y = {:.2}-{:.2}",
                        mouse_pos.x,
                        mouse_pos.y,
                        rhs_rect.min.x,
                        rhs_rect.max.x,
                        rhs_rect.min.y,
                        rhs_rect.max.y,
                    ),
                    egui::TextStyle::Monospace.resolve(&ctx.style()),
                    egui::Color32::RED,
                );
            }
        }

        fn draw_top_buttons(&mut self, ui: &mut egui::Ui) {
            ui.horizontal(|ui| {
                if ui
                    .add(egui::Button::new("🆕").min_size(egui::vec2(120.0, 50.0)))
                    .on_hover_text("Create a new configuration")
                    .clicked()
                {
                    log::info!("New button clicked");
                    // Implement New action logic here
                }

                if ui
                    .add(egui::Button::new("📂").min_size(egui::vec2(120.0, 50.0)))
                    .on_hover_text("Load an existing configuration")
                    .clicked()
                {
                    log::info!("Load button clicked");
                    // Implement Load action logic here
                }

                if ui
                    .add(egui::Button::new("💾").min_size(egui::vec2(120.0, 50.0)))
                    .on_hover_text("Save the current configuration")
                    .clicked()
                {
                    log::info!("Save button clicked");
                    self.save_character();
                }

                if ui
                    .add(egui::Button::new("🗑️").min_size(egui::vec2(120.0, 50.0)))
                    .on_hover_text("Clear all active nodes")
                    .clicked()
                {
                    log::info!("Clear button clicked");
                    self.clear_active_nodes();
                }
            });
        }

        fn search(&mut self, ui: &mut egui::Ui) {
            ui.separator();
            ui.heading("🔍 Search");

            if ui.text_edit_singleline(&mut self.search_query).changed() {
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

            if !self.search_query.is_empty() {
                ui.label("Search results:");
                for &node_id in &self.search_results {
                    if let Some(node) = self.passive_tree.nodes.get(&node_id) {
                        ui.horizontal(|ui| {
                            if ui.button(&node.name).clicked() {
                                self.go_to_node(node_id);
                                log::debug!("Navigated to node: {} ({})", node.name, node_id);
                            }
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button("📋").clicked() {
                                        ui.output_mut(|o| o.copied_text = node_id.to_string());
                                        log::info!("Copied Node ID {} to clipboard", node_id);
                                    }
                                },
                            );
                        });
                    }
                }
            }
        }

        fn pathing(&mut self, ui: &mut egui::Ui) {
            ui.separator();
            ui.heading("Node Configuration");

            // Start Node Configuration
            ui.horizontal(|ui| {
                ui.label("Start Node:");
                let mut start_node_str = self.start_node_id.to_string();
                if ui.text_edit_singleline(&mut start_node_str).changed() {
                    if let Ok(parsed) = start_node_str.parse::<usize>() {
                        if self.passive_tree.nodes.contains_key(&parsed) {
                            self.start_node_id = parsed;
                            log::info!("Start Node updated: {}", self.start_node_id);
                        } else {
                            log::warn!("Invalid Start Node ID: {}", parsed);
                        }
                    }
                }
            });

            // Target Node Configuration
            ui.horizontal(|ui| {
                ui.label("Target Node:");
                let mut target_node_str = self.target_node_id.to_string();
                if ui.text_edit_singleline(&mut target_node_str).changed() {
                    if let Ok(parsed) = target_node_str.parse::<usize>() {
                        if self.passive_tree.is_node_within_distance(
                            self.start_node_id,
                            parsed,
                            123,
                        ) {
                            self.target_node_id = parsed;
                            log::info!("Target Node updated: {}", self.target_node_id);
                        } else {
                            log::warn!(
                                "Node {} is not within 123 steps of Start Node {}",
                                parsed,
                                self.start_node_id
                            );
                        }
                    }
                }
            });
        }
    }
}

impl TreeVis<'_> {
    pub fn draw_color_and_highlights(&self, ctx: &egui::Context) {
        let painter = ctx.layer_painter(egui::LayerId::background());
        let active_color = parse_color(self.user_config.colors.get("yellow").unwrap());
        let search_color = parse_color(self.user_config.colors.get("purple").unwrap());
        let zoom = 1.0 + self.zoom;

        self.passive_tree.nodes.values().for_each(|node| {
            if node.active {
                let sx = self.world_to_screen_x(node.wx);
                let sy = self.world_to_screen_y(node.wy);

                let mut radius = Self::BASE_RADIUS / zoom;

                if node.is_notable {
                    radius *= Self::NOTABLE_MULTIPLIER;
                } else if !node.name.chars().any(|c| c.is_ascii_digit()) {
                    radius *= Self::NAMELESS_MULTIPLIER;
                }

                radius *= 0.05;

                painter.circle_stroke(
                    egui::pos2(sx, sy),
                    radius / self.current_zoom_level(),
                    egui::Stroke::new(3.0, active_color),
                );
            } else if self.search_results.contains(&node.node_id) {
                let sx = self.world_to_screen_x(node.wx);
                let sy = self.world_to_screen_y(node.wy);

                let mut radius = Self::BASE_RADIUS / zoom;

                if node.is_notable {
                    radius *= Self::NOTABLE_MULTIPLIER;
                } else if !node.name.chars().any(|c| c.is_ascii_digit()) {
                    radius *= Self::NAMELESS_MULTIPLIER;
                }

                radius *= 0.05;

                painter.circle_stroke(
                    egui::pos2(sx, sy),
                    radius / self.current_zoom_level(),
                    egui::Stroke::new(3.0, search_color),
                );
            }
        });
    }

    pub fn redraw_tree(&self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();

            self.draw_edges(painter);
            self.draw_nodes(painter);
        });
    }

    pub fn draw_edges(&self, painter: &egui::Painter) {
        let activated_edge_color =
            parse_color(self.user_config.colors.get("activated_edges").expect(
                "You MUST supply an .active_edges key in your toml with a valid colour",
            ));
        let default_edge_color = egui::Color32::GRAY;

        self.passive_tree.edges.iter().for_each(|edge| {
            if let (Some(source), Some(target)) = (
                self.passive_tree.nodes.get(&edge.start),
                self.passive_tree.nodes.get(&edge.end),
            ) {
                let sx = self.world_to_screen_x(source.wx);
                let sy = self.world_to_screen_y(source.wy);
                let tx = self.world_to_screen_x(target.wx);
                let ty = self.world_to_screen_y(target.wy);

                let color = if self.active_edges.contains(&(edge.start, edge.end)) {
                    activated_edge_color
                } else {
                    default_edge_color
                };

                painter.line_segment(
                    [egui::pos2(sx, sy), egui::pos2(tx, ty)],
                    egui::Stroke::new(1.5, color),
                );
            }
        });
    }

    pub fn draw_nodes(&self, painter: &egui::Painter) {
        let zoom = 1.0 + self.zoom; // Zoom level for scaling nodes

        self.passive_tree.nodes.values().for_each(|node| {
            let sx = self.world_to_screen_x(node.wx);
            let sy = self.world_to_screen_y(node.wy);

            let mut radius = Self::BASE_RADIUS / zoom;

            if node.is_notable {
                radius *= Self::NOTABLE_MULTIPLIER;
            }

            if !node.name.chars().any(|c| c.is_ascii_digit()) {
                radius *= Self::NAMELESS_MULTIPLIER;
            }

            let color = node.base_color(&self.user_config);

            painter.circle_filled(egui::pos2(sx, sy), radius, color);
        });
    }
}
