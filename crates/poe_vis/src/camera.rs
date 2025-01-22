//$ crates/poe_vis/src/camera.rs
use super::*;

// Helper Functions
impl TreeVis<'_> {
    pub(crate) const CAMERA_OFFSET: (f32, f32) = (-2_600.0, -1_300.0);
    pub(crate) const ZOOM_MIN: f32 = 0.0; // Minimum zoom level
    pub(crate) const ZOOM_MAX: f32 = 1.0; // Maximum zoom level
    pub(crate) const ZOOM_STEP: f32 = 0.0001; // Step size for zoom changes
    pub(crate) const DEFAULT_ZOOM_IN_WHEN_MOVE_TO_NODE: f32 = 0.50;

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
        // self.disable_fuzzy_search();
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

    pub fn world_to_screen_y(&self, wy: f32) -> f32 {
        (wy - self.camera.borrow().1) * *self.zoom.borrow() + 500.0
    }

    pub fn screen_to_world_x(&self, sx: f32) -> f32 {
        (sx - 500.0) / *self.zoom.borrow() + self.camera.borrow().0
    }

    pub fn screen_to_world_y(&self, sy: f32) -> f32 {
        (sy - 500.0) / *self.zoom.borrow() + self.camera.borrow().1
    }
}
