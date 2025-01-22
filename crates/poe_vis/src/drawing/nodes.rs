use poe_tree::config::parse_color;

use super::TreeVis;

impl TreeVis<'_> {
    //     pub(crate) const BASE_RADIUS: f32 = 5.0;
    //     pub(crate) const NOTABLE_MULTIPLIER: f32 = 1.5;
    //     pub(crate) const NAMELESS_MULTIPLIER: f32 = 1.0;
    //     pub(crate) const HIGHLIGHT_FOR_SEARCH: f32 = 0.18;
    //     pub(crate) const HIGHLIGHT_FOR_HOVER: f32 = 0.15;

    //     pub fn redraw_tree(&self, ctx: &egui::Context) {
    //         egui::CentralPanel::default().show(ctx, |ui| {
    //             let painter = ui.painter();

    //             // draw edges underneath where we place nodes.
    //             self.draw_edges(painter);
    //             self.restyle_edges(painter);

    //             // draw nodes second so they're ONTOP of the edges.
    //             self.draw_nodes(painter);
    //             self.restyle_nodes(painter);

    //             self.restyle_hovered_node(painter);
    //             // self.restyle_hovered_edge(painter); // needed?
    //         });
    //     }
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
    //     pub fn draw_nodes(&self, painter: &egui::Painter) {
    //         let zoom = 1.0 + *self.zoom.borrow_mut(); // Zoom level for scaling nodes

    //         self.passive_tree.nodes.values().for_each(|node| {
    //             let sx = self.world_to_screen_x(node.wx);
    //             let sy = self.world_to_screen_y(node.wy);

    //             let mut radius = Self::BASE_RADIUS * zoom;

    //             if node.is_notable {
    //                 radius *= Self::NOTABLE_MULTIPLIER;
    //             }

    //             if !node.name.chars().any(|c| c.is_ascii_digit()) {
    //                 radius *= Self::NAMELESS_MULTIPLIER;
    //             }

    //             let color = node.base_color(&self.user_config);

    //             painter.circle_filled(egui::pos2(sx, sy), radius, color);
    //         });
    //     }
}

// impl<'p> TreeVis<'p> {
//     // Adjusted radius calculation for consistent scaling
//     fn calculate_radius(&self, base: f32, notable: bool, nameless: bool) -> f32 {
//         let zoom_factor = self.current_zoom_level().max(0.1); // Prevent division by very small numbers
//         let mut radius = base / zoom_factor; // Adjust radius to stay consistent with zoom

//         if notable {
//             radius *= Self::NOTABLE_MULTIPLIER;
//         } else if nameless {
//             radius *= Self::NAMELESS_MULTIPLIER;
//         }

//         radius
//     }

//     // Restyle nodes with consistent scaling
//     pub fn restyle_nodes(&self, painter: &egui::Painter) {
//         let active_color = parse_color(self.user_config.colors.get("yellow").unwrap());
//         let search_color = parse_color(self.user_config.colors.get("purple").unwrap());

//         self.passive_tree.nodes.values().for_each(|node| {
//             let sx = self.world_to_screen_x(node.wx);
//             let sy = self.world_to_screen_y(node.wy);

//             if node.active {
//                 let radius = self.calculate_radius(Self::BASE_RADIUS, node.is_notable, false)
//                     * Self::HIGHLIGHT_FOR_HOVER;

//                 painter.circle_stroke(
//                     egui::pos2(sx, sy),
//                     radius,
//                     egui::Stroke::new(3.0, active_color),
//                 );
//             }

//             if self.search_results.contains(&node.node_id) {
//                 let radius = self.calculate_radius(Self::BASE_RADIUS, node.is_notable, false)
//                     * Self::HIGHLIGHT_FOR_SEARCH;

//                 painter.circle_stroke(
//                     egui::pos2(sx, sy),
//                     radius,
//                     egui::Stroke::new(3.0, search_color),
//                 );
//             }
//         });
//     }

//     pub fn restyle_hovered_node(&self, painter: &egui::Painter) {
//         if let Some(hovered_id) = self.hovered_node {
//             if let Some(node) = self.passive_tree.nodes.get(&hovered_id) {
//                 let hover_color = parse_color(self.user_config.colors.get("cyan").unwrap());

//                 let sx = self.world_to_screen_x(node.wx);
//                 let sy = self.world_to_screen_y(node.wy);

//                 let radius = self.calculate_radius(Self::BASE_RADIUS, node.is_notable, false) * 1.8; // Hover effect scaling

//                 painter.circle_stroke(
//                     egui::pos2(sx, sy),
//                     radius,
//                     egui::Stroke::new(3.0, hover_color),
//                 );
//             }
//         }
//     }
// }

impl<'p> TreeVis<'p> {
    pub(crate) const BASE_RADIUS: f32 = 5.0;
    pub(crate) const NOTABLE_DELTA: f32 = 0.5; // 50% increase for notable nodes
    pub(crate) const NAMELESS_DELTA: f32 = 0.2; // 20% increase for nameless nodes
    pub(crate) const HIGHLIGHT_SEARCH_DELTA: f32 = 0.3; // 30% increase for search highlights
    pub(crate) const HIGHLIGHT_HOVER_DELTA: f32 = 0.6; // 60% increase for hover highlights

    // Helper to calculate radius
    fn calculate_radius(&self, base: f32, notable: bool, nameless: bool, extra_delta: f32) -> f32 {
        let mut radius = base;

        if notable {
            radius += radius * Self::NOTABLE_DELTA;
        } else if nameless {
            radius += radius * Self::NAMELESS_DELTA;
        }

        radius + (radius * extra_delta)
    }

    // Helper to draw a node
    fn draw_node_at(
        &self,
        x: f32,
        y: f32,
        radius: f32,
        color: egui::Color32,
        painter: &egui::Painter,
        hollow: bool,
    ) {
        let position = egui::pos2(x, y);
        if hollow {
            painter.circle_stroke(position, radius, egui::Stroke::new(3.0, color));
        } else {
            painter.circle_filled(position, radius, color);
        }
    }

    // Draw base nodes
    pub fn draw_nodes(&self, painter: &egui::Painter) {
        let base_color = egui::Color32::GRAY;

        self.passive_tree.nodes.values().for_each(|node| {
            let x = self.world_to_screen_x(node.wx);
            let y = self.world_to_screen_y(node.wy);

            let radius = self.calculate_radius(
                Self::BASE_RADIUS,
                node.is_notable,
                !node.name.chars().any(|c| c.is_ascii_digit()),
                0.0,
            );

            self.draw_node_at(x, y, radius, base_color, painter, false);
        });
    }

    // Restyle active, hovered, and searched nodes
    pub fn restyle_nodes(&self, painter: &egui::Painter) {
        let active_color = parse_color(self.user_config.colors.get("yellow").unwrap());
        let search_color = parse_color(self.user_config.colors.get("purple").unwrap());

        // Active nodes
        self.passive_tree.nodes.values().for_each(|node| {
            if node.active {
                let x = self.world_to_screen_x(node.wx);
                let y = self.world_to_screen_y(node.wy);

                let radius = self.calculate_radius(
                    Self::BASE_RADIUS,
                    node.is_notable,
                    false,
                    Self::HIGHLIGHT_HOVER_DELTA,
                );

                self.draw_node_at(x, y, radius, active_color, painter, true);
            }
        });

        // Search result nodes
        self.search_results.iter().for_each(|&node_id| {
            if let Some(node) = self.passive_tree.nodes.get(&node_id) {
                let x = self.world_to_screen_x(node.wx);
                let y = self.world_to_screen_y(node.wy);

                let radius = self.calculate_radius(
                    Self::BASE_RADIUS,
                    node.is_notable,
                    false,
                    Self::HIGHLIGHT_SEARCH_DELTA,
                );

                self.draw_node_at(x, y, radius, search_color, painter, true);
            }
        });
    }

    // Restyle hovered node
    pub fn restyle_hovered_node(&self, painter: &egui::Painter) {
        if let Some(hovered_id) = self.hovered_node {
            if let Some(node) = self.passive_tree.nodes.get(&hovered_id) {
                let hover_color = parse_color(self.user_config.colors.get("cyan").unwrap());
                let x = self.world_to_screen_x(node.wx);
                let y = self.world_to_screen_y(node.wy);

                let radius = self.calculate_radius(
                    Self::BASE_RADIUS,
                    node.is_notable,
                    false,
                    Self::HIGHLIGHT_HOVER_DELTA,
                );

                self.draw_node_at(x, y, radius, hover_color, painter, true);
            }
        }
    }

    pub fn redraw_tree(&self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();

            // Draw edges first
            self.draw_edges(painter);
            self.restyle_edges(painter);

            // Draw nodes on top of edges
            self.draw_nodes(painter);
            self.restyle_nodes(painter);

            // Draw hovered node last
            self.restyle_hovered_node(painter);
        });
    }
}
