use poe_tree::config::parse_color;

use super::TreeVis;

impl TreeVis<'_> {
    pub(crate) const BASE_RADIUS: f32 = 5.0;
    pub(crate) const NOTABLE_MULTIPLIER: f32 = 1.5;
    pub(crate) const NAMELESS_MULTIPLIER: f32 = 1.0;
    pub(crate) const HIGHLIGHT_FOR_SEARCH: f32 = 0.18;
    pub(crate) const HIGHLIGHT_FOR_HOVER: f32 = 0.15;

    // Restyles active, and 'searched' nodes.
    pub fn restyle_nodes(&self, painter: &egui::Painter) {
        let active_color = parse_color(self.user_config.colors.get("yellow").unwrap());
        let search_color = parse_color(self.user_config.colors.get("purple").unwrap());
        let zoom = 1.0 + *self.zoom.borrow_mut();

        self.passive_tree.nodes.values().for_each(|node| {
            // self.active, made so by a click, unclick (deactivates), or by loading characters' data..
            if node.active {
                let sx = self.world_to_screen_x(node.wx);
                let sy = self.world_to_screen_y(node.wy);

                let mut radius = Self::BASE_RADIUS * zoom;

                if node.is_notable {
                    radius *= Self::NOTABLE_MULTIPLIER;
                } else if !node.name.chars().any(|c| c.is_ascii_digit()) {
                    radius *= Self::NAMELESS_MULTIPLIER;
                }

                radius *= Self::HIGHLIGHT_FOR_HOVER;

                painter.circle_stroke(
                    egui::pos2(sx, sy),
                    radius / self.current_zoom_level(),
                    egui::Stroke::new(3.0, active_color),
                );
            }

            if self.search_results.contains(&node.node_id) {
                let sx = self.world_to_screen_x(node.wx);
                let sy = self.world_to_screen_y(node.wy);

                let mut radius = Self::BASE_RADIUS * zoom;

                if node.is_notable {
                    radius *= Self::NOTABLE_MULTIPLIER;
                } else if !node.name.chars().any(|c| c.is_ascii_digit()) {
                    radius *= Self::NAMELESS_MULTIPLIER;
                }

                // we want this to always be the largest highlight, even visible on active etc.
                radius *= Self::HIGHLIGHT_FOR_SEARCH;

                painter.circle_stroke(
                    egui::pos2(sx, sy),
                    radius / self.current_zoom_level(),
                    egui::Stroke::new(3.0, search_color),
                );
            }
        });
    }
    pub fn restyle_hovered_node(&self, painter: &egui::Painter) {
        // should be impossible
        let Some(hovered_id) = self.hovered_node else {
            return;
        };

        let Some(node) = self.passive_tree.nodes.get(&hovered_id) else {
            // This should be impossible
            log::error!("Somehow we've been 'hovered' on a ID:{} that doesn't exist in our self.passive_tree.nodes", self.hovered_node.unwrap());
            return;
        };

        let hover_color = parse_color(self.user_config.colors.get("cyan").unwrap());
        let zoom = 1.0 + *self.zoom.borrow_mut();

        let sx = self.world_to_screen_x(node.wx);
        let sy = self.world_to_screen_y(node.wy);

        let mut radius = Self::BASE_RADIUS * zoom;

        if node.is_notable {
            radius *= Self::NOTABLE_MULTIPLIER;
        } else if !node.name.chars().any(|c| c.is_ascii_digit()) {
            radius *= Self::NAMELESS_MULTIPLIER;
        }

        radius *= 0.055 * 1.8; // Increase radius by 1.8x

        // Draw a hollow circle (stroke only, no fill)
        painter.circle_stroke(
            egui::pos2(sx, sy),
            radius / self.current_zoom_level(),
            egui::Stroke::new(3.0, hover_color),
        );
    }

    pub fn redraw_tree(&self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();

            // draw edges underneath where we place nodes.
            self.draw_edges(painter);
            self.restyle_edges(painter);

            // draw nodes second so they're ONTOP of the edges.
            self.draw_nodes(painter);
            self.restyle_nodes(painter);

            self.restyle_hovered_node(painter);
            // self.restyle_hovered_edge(painter); // needed?
        });
    }
    pub fn restyle_edges(&self, painter: &egui::Painter) {
        let activated_edge_color = parse_color(
            self.user_config
                .colors
                .get("activated_edges")
                .expect("You MUST supply an .active_edges key in your toml with a valid colour"),
        );

        self.active_edges.iter().for_each(|edge| {
            if let (Some(source), Some(target)) = (
                self.passive_tree.nodes.get(&edge.0),
                self.passive_tree.nodes.get(&edge.1),
            ) {
                let sx = self.world_to_screen_x(source.wx);
                let sy = self.world_to_screen_y(source.wy);
                let tx = self.world_to_screen_x(target.wx);
                let ty = self.world_to_screen_y(target.wy);

                painter.line_segment(
                    [egui::pos2(sx, sy), egui::pos2(tx, ty)],
                    egui::Stroke::new(1.8, activated_edge_color),
                );
            }
        });
    }
    pub fn draw_edges(&self, painter: &egui::Painter) {
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

                painter.line_segment(
                    [egui::pos2(sx, sy), egui::pos2(tx, ty)],
                    egui::Stroke::new(1.5, default_edge_color),
                );
            }
        });
    }
    pub fn draw_nodes(&self, painter: &egui::Painter) {
        let zoom = 1.0 + *self.zoom.borrow_mut(); // Zoom level for scaling nodes

        self.passive_tree.nodes.values().for_each(|node| {
            let sx = self.world_to_screen_x(node.wx);
            let sy = self.world_to_screen_y(node.wy);

            let mut radius = Self::BASE_RADIUS * zoom;

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
