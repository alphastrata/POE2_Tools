//!$ crates/poe_vis/src/drawing/top_menu.rs
use rfd::FileDialog;
use std::path::PathBuf;

use super::TreeVis;
impl TreeVis<'_> {
    pub(crate) fn draw_top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // File menu
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        if let Some(target) = self.open_file_dialog() {
                            // TODO: Handle the opened file
                            log::info!("Opened file: {:?}", target);

                            self.load_character(target);
                            assert_eq!(self.user_config.character.name, "jengablox");
                        }
                        ui.close_menu();
                    }
                    if ui.button("Save").clicked() {
                        log::info!("Save clicked");
                        ui.close_menu();
                    }
                    if ui.button("Save As").clicked() {
                        log::info!("Save As clicked");
                        ui.close_menu();
                    }
                });

                // View menu remains unchanged
                ui.menu_button("View", |ui| {
                    // if ui.button("Reset Zoom").clicked() {
                    //     self.reset_zoom();
                    //     ui.close_menu();
                    // }
                    // if ui.button("Zoom In").clicked() {
                    //     self.zoom_in();
                    //     ui.close_menu();
                    // }
                    // if ui.button("Zoom Out").clicked() {
                    //     self.zoom_out();
                    //     ui.close_menu();
                    // }
                });
            });
        });
    }

    fn open_file_dialog(&self) -> Option<PathBuf> {
        FileDialog::new()
            .add_filter("TOML files", &["toml"])
            .set_directory(PathBuf::from("./data"))
            .pick_file()
    }
}
