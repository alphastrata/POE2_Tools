//!$ crates/poe_vis/src/camera.rs
use poe_tree::nodes::PoeNode;

use super::*;

// Helper Functions
impl TreeVis<'_> {
    pub(crate) const CAMERA_OFFSET: (f32, f32) = (-2_600.0, -1_300.0);
    pub(crate) const ZOOM_MIN: f32 = 0.0; // Minimum zoom level
    pub(crate) const ZOOM_MAX: f32 = 1.3; // Maximum zoom level
    pub(crate) const ZOOM_STEP: f32 = 0.0008; // Step size for zoom changes
    pub(crate) const DEFAULT_ZOOM_IN_WHEN_MOVE_TO_NODE: f32 = 0.38;
    pub(crate) const DEFAULT_STARTING_CAMERA_ZOOM: f32 = 0.09;

    pub fn move_camera_to_node(&self, node_id: u32) {
        if let Some(node) = self.passive_tree.nodes.get(&node_id) {
            let mut camera = self.camera.borrow_mut();
            camera.0 = node.wx;
            camera.1 = node.wy;

            self.set_zoom_level(Self::DEFAULT_ZOOM_IN_WHEN_MOVE_TO_NODE);

            log::debug!(
                "Camera centered on node ID: {} at world position: ({:.2}, {:.2})",
                node_id,
                node.wx,
                node.wy
            );
        }
    }
    pub fn go_to_node(&self, id: u32) {
        self.move_camera_to_node(id);
        self.close_fuzzy_search();
    }

    pub fn current_zoom_level(&self) -> f32 {
        *self.zoom.borrow()
    }

    pub fn set_zoom_level(&self, zoom: f32) {
        *self.zoom.borrow_mut() = zoom;
    }

    /// Translate the camera based on mouse drag input.
    pub fn translate_camera(&mut self, dx: f32, dy: f32) {
        let mut camera = self.camera.borrow_mut();
        let zoom = self.zoom.borrow();
        camera.0 += dx / *zoom; // Adjust for current zoom level
        camera.1 += dy / *zoom;
    }

    /// Adjust the zoom level based on raw scroll input.
    pub fn adjust_zoom(&mut self, scroll: f32, mouse_pos: egui::Pos2) {
        let new_zoom =
            (*self.zoom.borrow() + scroll * Self::ZOOM_STEP).clamp(Self::ZOOM_MIN, Self::ZOOM_MAX);

        if (new_zoom - *self.zoom.borrow()).abs() > f32::EPSILON {
            // Calculate the scaling factor
            let _scale = new_zoom / *self.zoom.borrow();

            // Adjust the camera to center zooming around the mouse position
            let screen_center_x = self.camera.borrow().0 + mouse_pos.x / *self.zoom.borrow();
            let screen_center_y = self.camera.borrow().1 + mouse_pos.y / *self.zoom.borrow();

            let new_camera_x = screen_center_x - mouse_pos.x / new_zoom;
            let new_camera_y = screen_center_y - mouse_pos.y / new_zoom;

            // Update camera and zoom
            *self.camera.borrow_mut() = (new_camera_x, new_camera_y);
            *self.zoom.borrow_mut() = new_zoom;
        }
    }

    pub fn world_to_screen_x(&self, wx: f32) -> f32 {
        (wx - self.camera.borrow().0) * *self.zoom.borrow() + 500.0
    }
    pub fn screen_to_world_x(&self, sx: f32) -> f32 {
        (sx - 500.0) / *self.zoom.borrow() + self.camera.borrow().0
    }

    pub fn world_to_screen_y(&self, wy: f32) -> f32 {
        (wy - self.camera.borrow().1) * *self.zoom.borrow() + 500.0
    }
    pub fn screen_to_world_y(&self, sy: f32) -> f32 {
        (sy - 500.0) / *self.zoom.borrow() + self.camera.borrow().1
    }

    // pub fn world_to_screen_y(&self, wy: f32) -> f32 {
    //     500.0 - (wy - self.camera.borrow().1) * *self.zoom.borrow()
    // }

    // pub fn screen_to_world_y(&self, sy: f32) -> f32 {
    //     (500.0 - sy) / *self.zoom.borrow() + self.camera.borrow().1
    // }

    pub fn cam_xy(&self) -> (f32, f32) {
        (*self.camera.borrow()).clone()
    }
}

// Culling experiments:
impl TreeVis<'_> {
    /// Get the world-space bounds of the visible area
    pub fn get_camera_view_rect(&self, ctx: &egui::Context) -> egui::Rect {
        egui::Rect::from_min_max(
            egui::pos2(
                self.camera.borrow().0
                    - (ctx.screen_rect().width() / (2.0 * self.current_zoom_level()))
                    - ((ctx.screen_rect().width() / (2.0 * self.current_zoom_level())) * 0.30),
                self.camera.borrow().1
                    - (ctx.screen_rect().height() / (2.0 * self.current_zoom_level()))
                    - ((ctx.screen_rect().height() / (2.0 * self.current_zoom_level())) * 0.30),
            ),
            egui::pos2(
                self.camera.borrow().0
                    + (ctx.screen_rect().width() / (2.0 * self.current_zoom_level()))
                    + ((ctx.screen_rect().width() / (2.0 * self.current_zoom_level())) * 0.30),
                self.camera.borrow().1
                    + (ctx.screen_rect().height() / (2.0 * self.current_zoom_level()))
                    + ((ctx.screen_rect().height() / (2.0 * self.current_zoom_level())) * 0.30),
            ),
        )
    }

    /// Find the world-space coordinates of the farthest visible nodes
    pub fn get_farthest_visible_nodes(
        &self,
        ctx: &egui::Context,
    ) -> impl Iterator<Item = &PoeNode> + '_ {
        let view_rect = self.get_camera_view_rect(ctx);

        self.passive_tree.nodes.values().filter(move |node| {
            let node_pos = egui::pos2(node.wx, node.wy);
            view_rect.contains(node_pos)
        })
    }
}
