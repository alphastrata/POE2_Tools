//!$ crates/poe_vis/src/drawing/rhs_menu.rs
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

                ui.separator();
                ui.heading("Character");
                self.display_character_section(ui);
            })
            .response
            .rect; // Capture the rectangle of the RHS menu

        // Log the RHS menu dimensions
        log::trace!(
            "RHS menu rect: x = {:.2}, y = {:.2}, width = {:.2}, height = {:.2}",
            rhs_rect.min.x,
            rhs_rect.min.y,
            rhs_rect.width(),
            rhs_rect.height(),
        );

        // Get and log the mouse position
        if let Some(mouse_pos) = ctx.input(|input| input.pointer.hover_pos()) {
            log::trace!(
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

    fn display_character_section(&mut self, ui: &mut egui::Ui) {
        if let Some(character) = &self.current_character {
            ui.label(format!("Class: {}", character.character_class));
            ui.label(format!("Starting node: {}", character.starting_node));
            ui.label(format!("Name: {}", character.name));
            ui.label(format!("Activated Nodes: {}", self.active_nodes.len()));
            ui.label(format!("Date Created: {}", character.date_created));
            ui.label(format!("Level: {}", character.level));
        } else {
            ui.label("No character loaded.");
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
                self.clear_active_nodes_and_edges();
            }
        });
    }

    fn search(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.heading("🔍 Search");

        if ui.text_edit_singleline(&mut self.search_query).changed() {
            if !self.search_query.is_empty() {
                self.search_results = self.passive_tree.fuzzy_search_nodes(&self.search_query);
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
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("📋").clicked() {
                                ui.output_mut(|o| o.copied_text = node_id.to_string());
                                log::info!("Copied Node ID {} to clipboard", node_id);
                            }
                        });
                    });
                }
            }
        }
    }
}
