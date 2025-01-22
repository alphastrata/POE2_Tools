use std::fmt;

use crate::TreeVis;

/// A struct representing the content displayed in the debug bar.
pub struct BottomDebugDisplay {
    pub mouse_pos: (f32, f32),
    pub mouse_distance_from_camera_pos: f32,
    pub zoom_level: f32,
    pub hovered_node_id: Option<u32>,
    pub hovered_node_name: Option<String>,
    pub node_dist_from_origin: Option<f32>,
    pub world_mouse_pos: (f32, f32),
    pub camera_pos: (f32, f32),
}

impl BottomDebugDisplay {
    /// Create a new `BottomDebugDisplay` instance from the `TreeVis` state.
    pub fn from_tree_vis(tree_vis: &TreeVis, ctx: &egui::Context) -> Self {
        // Get mouse position
        let mouse_pos = ctx.input(|input| input.pointer.hover_pos().unwrap_or_default());

        // Convert mouse position to world coordinates
        let world_mouse_x = tree_vis.screen_to_world_x(mouse_pos.x);
        let world_mouse_y = tree_vis.screen_to_world_y(mouse_pos.y);

        // Get camera position
        let camera_pos = *tree_vis.camera.borrow();

        // Calculate distance from mouse to camera position
        let mouse_distance_from_camera_pos = ((world_mouse_x - camera_pos.0).powi(2)
            + (world_mouse_y - camera_pos.1).powi(2))
        .sqrt();

        // Get zoom level
        let zoom_level = *tree_vis.zoom.borrow();

        // Get hovered node info
        let (hovered_node_id, hovered_node_name, node_dist_from_origin) =
            if let Some(hovered_node_id) = tree_vis.hovered_node {
                if let Some(node) = tree_vis.passive_tree.nodes.get(&hovered_node_id) {
                    let dist = (node.wx.powi(2) + node.wy.powi(2)).sqrt();
                    (Some(hovered_node_id), Some(node.name.clone()), Some(dist))
                } else {
                    (Some(hovered_node_id), None, None)
                }
            } else {
                (None, None, None)
            };

        Self {
            mouse_pos: (mouse_pos.x, mouse_pos.y),
            mouse_distance_from_camera_pos,
            zoom_level,
            hovered_node_id,
            hovered_node_name,
            node_dist_from_origin,
            world_mouse_pos: (world_mouse_x, world_mouse_y),
            camera_pos,
        }
    }
}

impl fmt::Display for BottomDebugDisplay {
    /// Render the `BottomDebugDisplay` contents to the UI.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "MOUSE: ({:.2}, {:.2}) | ZOOM: {:.2} | W_MOUSE: ({:.2}, {:.2}) | CAM: ({:.2}, {:.2}) | CAM_DIST: {:.2}",
            self.mouse_pos.0, self.mouse_pos.1, self.zoom_level,
            self.world_mouse_pos.0, self.world_mouse_pos.1,
            self.camera_pos.0, self.camera_pos.1,
            self.mouse_distance_from_camera_pos,
        )?;

        writeln!(
            f,
            "Hovered Node: {} | Distance from Origin: {:.2}",
            self.hovered_node_name.as_deref().unwrap_or("None"),
            self.node_dist_from_origin.unwrap_or(f32::NAN),
        )
    }
}

impl TreeVis<'_> {
    pub fn draw_debug_bar(&mut self, ctx: &egui::Context) {
        let debug_display = BottomDebugDisplay::from_tree_vis(self, ctx);

        egui::TopBottomPanel::bottom("debug_panel").show(ctx, |ui| {
            ui.label(format!("{}", debug_display));
        });

        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            "crosshair".into(),
        ));

        // Get camera world and screen positions
        let (cam_x, cam_y) = *self.camera.borrow();
        let screen_x = self.world_to_screen_x(cam_x);
        let screen_y = self.world_to_screen_y(cam_y);

        // log::debug!(
        //     "Camera world position: ({:.2}, {:.2}), Screen position: ({:.2}, {:.2})",
        //     cam_x,
        //     cam_y,
        //     screen_x,
        //     screen_y
        // );
        let view_rect = self.get_camera_view_rect(ctx);
        log::debug!("Camera View Rect: {:?}", view_rect);

        self.draw_crosshair(ctx, &painter);
    }

    pub fn draw_crosshair(&self, ctx: &egui::Context, painter: &egui::Painter) {
        let (cam_x, cam_y) = *self.camera.borrow();

        // Transform world-space (0, 0) to screen-space
        let screen_x = self.world_to_screen_x(0.0);
        let screen_y = self.world_to_screen_y(0.0);

        log::debug!(
            "World (0, 0) transformed to screen ({:.2}, {:.2}), Camera: ({:.2}, {:.2})",
            screen_x,
            screen_y,
            cam_x,
            cam_y
        );

        let crosshair_color = egui::Color32::RED;
        let crosshair_size = 10.0;

        // Draw horizontal and vertical lines for the crosshair
        painter.line_segment(
            [
                egui::pos2(screen_x - crosshair_size, screen_y),
                egui::pos2(screen_x + crosshair_size, screen_y),
            ],
            (1.0, crosshair_color),
        );

        painter.line_segment(
            [
                egui::pos2(screen_x, screen_y - crosshair_size),
                egui::pos2(screen_x, screen_y + crosshair_size),
            ],
            (1.0, crosshair_color),
        );
        self.draw_centered_cull_region(ctx, painter);
    }

    pub fn draw_centered_cull_region(&self, ctx: &egui::Context, painter: &egui::Painter) {
        let (cam_x, cam_y) = self.cam_xy();
        let cull_min = egui::pos2(cam_x, cam_y);

        let max_screen_x = cam_x + Self::CAMERA_OFFSET.0.abs();
        let max_screen_y = cam_y + Self::CAMERA_OFFSET.1.abs();

        let cull_max = egui::pos2(max_screen_x, max_screen_y);

        // Draw the cull region as a rectangle
        painter.rect_stroke(
            egui::Rect::from_min_max(cull_min, cull_max),
            0.0, // No rounding
            egui::Stroke::new(1.0, egui::Color32::GREEN),
        );

        log::debug!(
            "Cull Region: Min ({:.2}, {:.2}), Max ({:.2}, {:.2})",
            cull_min.x,
            cull_min.y,
            cull_max.x,
            cull_max.y,
        );
    }
}
