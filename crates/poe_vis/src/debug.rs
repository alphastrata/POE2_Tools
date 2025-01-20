//$ crates/poe_vis/src/debug.rs
use super::*;

// DEBUG BAR
impl TreeVis<'_> {
    /// Draw the debug information bar
    pub fn draw_debug_bar(&self, ctx: &egui::Context) {
        let (
            mouse_info,
            zoom_info,
            hovered_node_info,
            node_dist_from_origin,
            world_mouse_pos,
            camera_pos,
        ) = self.get_debug_bar_contents(ctx);

        egui::TopBottomPanel::bottom("debug_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(mouse_info);
                ui.separator();
                ui.label(zoom_info);
                ui.separator();
                ui.label(hovered_node_info);
                ui.separator();
                ui.label(node_dist_from_origin);
                ui.separator();
                ui.label(world_mouse_pos);
                ui.separator();
                ui.label(camera_pos);
            });
        });
    }

    /// Precompute debug bar contents to avoid borrow conflicts
    pub fn get_debug_bar_contents(
        &self,
        ctx: &egui::Context,
    ) -> (String, String, String, String, String, String) {
        // Get mouse position
        let mouse_pos = ctx.input(|input| input.pointer.hover_pos().unwrap_or_default());
        let mouse_info = format!("Mouse: ({:.2}, {:.2})", mouse_pos.x, mouse_pos.y);

        // Convert mouse position to world coordinates
        let world_mouse_x = self.screen_to_world_x(mouse_pos.x);
        let world_mouse_y = self.screen_to_world_y(mouse_pos.y);
        let world_mouse_pos = format!("World Mouse: ({:.2}, {:.2})", world_mouse_x, world_mouse_y);

        // Get camera position
        let (camera_x, camera_y) = *self.camera.borrow();
        let camera_pos = format!("Camera: ({:.2}, {:.2})", camera_x, camera_y);

        // Get zoom level
        let zoom_info = format!("Zoom: {:.2}", self.zoom);

        // Get hovered node info
        let hovered_node_info = if let Some(hovered_node_id) = self.hovered_node {
            if let Some(node) = self.passive_tree.nodes.get(&hovered_node_id) {
                format!("Hovered Node: {:?}", node)
            } else {
                format!(
                    "Hovered Node: {} (not found in passive_tree)",
                    hovered_node_id
                )
            }
        } else {
            "Hovered Node: None".to_string()
        };

        // Get distance from (0,0) for hovered node
        let node_dist_from_origin = if let Some(hovered_node_id) = self.hovered_node {
            if let Some(node) = self.passive_tree.nodes.get(&hovered_node_id) {
                let dist = (node.wx.powi(2) + node.wy.powi(2)).sqrt();
                format!("Distance from Origin: {:.2}", dist)
            } else {
                "Distance from Origin: N/A".to_string()
            }
        } else {
            "Distance from Origin: N/A".to_string()
        };

        (
            mouse_info,
            zoom_info,
            hovered_node_info,
            node_dist_from_origin,
            world_mouse_pos,
            camera_pos,
        )
    }
}
