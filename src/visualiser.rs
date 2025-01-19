//$ src\visualiser.rs
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    default::Default,
    ops::ControlFlow,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::Instant,
};

use crate::{config::UserConfig, data::poe_tree::PassiveTree};
use crate::{
    config::{parse_color, UserCharacter},
    data::poe_tree::type_wrappings::NodeId,
};
static ACTIVE_NODE_COUNT: AtomicUsize = AtomicUsize::new(0);

impl eframe::App for TreeVis<'_> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // data updates:
        self.check_and_activate_nodes();
        self.check_and_activate_edges();

        // IO
        self.handle_mouse(ctx);

        //DEBUG:
        self.draw_debug_bar(ctx);

        // Example: Process node hovering
        if let Some(hovered_node_id) = self.get_hovered_node(ctx) {
            self.hover_node(hovered_node_id);
        }

        // Example: Check and activate nodes if target node changes
        if let Some(target_node_id) = self.get_target_node() {
            self.select_node(target_node_id);
        }

        ctx.input(|input| {
            if let Some(hovered) = self.hovered_node {
                if input.pointer.primary_clicked() {
                    self.click_node(hovered);
                }
            }

            if input.key_pressed(egui::Key::Escape) {
                std::process::exit(0);
            }
        });

        // drawing
        self.redraw_tree(ctx);
        // TODO: maybe we highlight in the redraw_tree() call?
        self.draw_color_and_highlights(ctx);

        //TODO: draw rhs menu
        self.draw_rhs_menu(ctx);

        //todo: draw top menu (open tree, char etc..)
    }
}

impl TreeVis<'_> {
    pub fn set_start_node(&mut self, ctx: &egui::Context) {
        if let Some((&closest_id, closest_node)) =
            self.passive_tree.nodes.iter().min_by(|(_, a), (_, b)| {
                let dist_a = (a.wx.powi(2) + a.wy.powi(2)).sqrt();
                let dist_b = (b.wx.powi(2) + b.wy.powi(2)).sqrt();
                dist_a
                    .partial_cmp(&dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        {
            self.start_node_id = closest_id;
            log::info!("Start node set to ID: {}", closest_id);

            // Draw a small white triangle at the start node
            let painter = ctx.layer_painter(egui::LayerId::background());
            let sx = self.world_to_screen_x(closest_node.wx);
            let sy = self.world_to_screen_y(closest_node.wy);

            painter.add(egui::Shape::convex_polygon(
                vec![
                    egui::pos2(sx, sy - 5.0),       // Top point of the triangle
                    egui::pos2(sx - 4.0, sy + 3.0), // Bottom-left point
                    egui::pos2(sx + 4.0, sy + 3.0), // Bottom-right point
                ],
                egui::Color32::WHITE,
                egui::Stroke::NONE,
            ));
        }
    }
}
pub struct TreeVis<'p> {
    camera: RefCell<(f32, f32)>,
    zoom: f32,
    passive_tree: &'p mut PassiveTree,
    hovered_node: Option<usize>,

    // Fuzzy-search-related
    fuzzy_search_open: AtomicBool,
    search_query: String,
    search_results: Vec<usize>,

    // Path-finder-related
    start_node_id: usize,
    target_node_id: usize,
    highlighted_path: Vec<usize>,

    /// Store edges of the current path
    // NOTE: mostly used for drawing.
    active_edges: HashSet<(usize, usize)>,
    active_nodes: HashSet<usize>,

    // Config-driven colours
    current_character: Option<UserCharacter>,
    last_save_time: std::time::Instant,

    user_config: UserConfig,

    /// Mapped controls from self.user_config
    #[allow(unused)]
    controls: HashMap<String, egui::Key>,
}

// Helper Functions
impl<'p> TreeVis<'p> {
    // Camera consts
    const ZOOM_MIN: f32 = 0.0; // Minimum zoom level
    const ZOOM_MAX: f32 = 1.0; // Maximum zoom level
    const ZOOM_STEP: f32 = 0.0001; // Step size for zoom changes

    // Node size constants
    const BASE_RADIUS: f32 = 8.0;
    const NOTABLE_MULTIPLIER: f32 = 1.5; // Scale notable nodes
    const NAMELESS_MULTIPLIER: f32 = 1.0; // Scale nameless nodes

    fn current_zoom_level(&self) -> f32 {
        self.zoom
    }

    #[allow(unused)]
    fn enable_fuzzy_search(&self) {
        self.fuzzy_search_open.store(true, Ordering::Relaxed);
    }

    #[allow(unused)]
    fn disable_fuzzy_search(&self) {
        self.fuzzy_search_open.store(false, Ordering::Relaxed);
    }

    #[allow(unused)]
    fn is_fuzzy_search_open(&self) -> bool {
        self.fuzzy_search_open.load(Ordering::Relaxed)
    }

    const CAMERA_OFFSET: (f32, f32) = (-2_600.0, -1_300.0);
    pub fn new(
        passive_tree: &'p mut PassiveTree,
        user_config: UserConfig,
        current_character: Option<UserCharacter>,
    ) -> Self {
        Self {
            camera: RefCell::new(Self::CAMERA_OFFSET),
            zoom: 0.09,
            passive_tree,
            hovered_node: None, // No node hovered initially

            // Fuzzy-search-related
            fuzzy_search_open: AtomicBool::new(false), // Search not open initially
            search_query: String::new(),               // Empty search query
            search_results: Vec::new(),                // No search results initially

            // Path-finder-related
            start_node_id: 0,             // Default to the root or initial node
            target_node_id: 0,            // Default to no target node
            highlighted_path: Vec::new(), // No path initially
            active_edges: HashSet::new(), // No edges highlighted initially
            active_nodes: HashSet::new(),

            current_character,
            last_save_time: std::time::Instant::now(), // Set to the current time

            user_config,
            controls: HashMap::new(),
        }
    }

    fn move_camera_to_node(&self, node_id: usize) {
        if let Some(node) = self.passive_tree.nodes.get(&node_id) {
            let mut camera = self.camera.borrow_mut();
            camera.0 = node.wx as f32;
            camera.1 = node.wy as f32;

            log::debug!(
                "Camera centered on node ID: {} at world position: ({:.2}, {:.2})",
                node_id,
                node.wx,
                node.wy
            );
        }
    }
    fn go_to_node(&self, id: usize) {
        self.move_camera_to_node(id);
        self.disable_fuzzy_search();
    }

    fn save_character(&mut self) {
        if let Some(character) = &self.current_character {
            character.save_to_toml("data/last_character.toml");
            self.last_save_time = std::time::Instant::now();
        }
    }

    #[allow(unused)]
    fn auto_save_character(&mut self) {
        if let Some(character) = &self.current_character {
            if self.last_save_time.elapsed().as_secs() >= 5 {
                self.save_character();
            }
        }
    }

    pub fn load_character(&mut self, path: &str) {
        self.current_character = UserCharacter::load_from_toml(path);
    }

    fn handle_mouse(&mut self, ctx: &egui::Context) {
        ctx.input(|input| {
            if input.raw_scroll_delta.y != 0.0 {
                let raw_scroll = input.raw_scroll_delta.y;
                let mouse_pos = input.pointer.hover_pos().unwrap_or_default();
                self.adjust_zoom(raw_scroll, mouse_pos);
            }

            // Handle click-and-drag for camera translation
            if input.pointer.primary_down() {
                let mouse_delta = input.pointer.delta();
                self.translate_camera(-mouse_delta.x, -mouse_delta.y);
            }

            // Update hovered node
            if let Some(mouse_pos) = input.pointer.hover_pos() {
                self.update_hovered_node(mouse_pos);
            }
        });
    }

    /// Translate the camera based on mouse drag input.
    fn translate_camera(&mut self, dx: f32, dy: f32) {
        let mut camera = self.camera.borrow_mut();
        camera.0 += dx / self.zoom; // Adjust for current zoom level
        camera.1 += dy / self.zoom;
    }

    /// Update the hovered node based on mouse position.
    fn update_hovered_node(&mut self, mouse_pos: egui::Pos2) {
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
    /// Adjust the zoom level based on raw scroll input.
    fn adjust_zoom(&mut self, scroll: f32, mouse_pos: egui::Pos2) {
        let new_zoom = (self.zoom + scroll * Self::ZOOM_STEP).clamp(Self::ZOOM_MIN, Self::ZOOM_MAX);

        if (new_zoom - self.zoom).abs() > f32::EPSILON {
            // Calculate the scaling factor
            let _scale = new_zoom / self.zoom;

            // Adjust the camera to center zooming around the mouse position
            let screen_center_x = self.camera.borrow().0 + mouse_pos.x / self.zoom;
            let screen_center_y = self.camera.borrow().1 + mouse_pos.y / self.zoom;

            let new_camera_x = screen_center_x - mouse_pos.x / new_zoom;
            let new_camera_y = screen_center_y - mouse_pos.y / new_zoom;

            // Update camera and zoom
            *self.camera.borrow_mut() = (new_camera_x, new_camera_y);
            self.zoom = new_zoom;
        }
    }

    fn world_to_screen_x(&self, wx: f64) -> f32 {
        (wx as f32 - self.camera.borrow().0) * self.zoom + 500.0
    }

    fn world_to_screen_y(&self, wy: f64) -> f32 {
        (wy as f32 - self.camera.borrow().1) * self.zoom + 500.0
    }

    fn click_node(&mut self, node_id: NodeId) {
        if let Some(node) = self.passive_tree.nodes.get_mut(&node_id) {
            node.active = !node.active;
            if node.active {
                if !self.highlighted_path.contains(&node_id) {
                    self.highlighted_path.push(node_id);
                }
            } else {
                self.highlighted_path.retain(|&id| id != node_id);
            }
            log::debug!("Node {} active state toggled to: {}", node_id, node.active);
            log::debug!("Current path nodes: {:?}", self.highlighted_path);
        }
    }

    fn draw_color_and_highlights(&self, ctx: &egui::Context) {
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

    fn draw_edges(&self, painter: &egui::Painter) {
        let activated_edge_color = parse_color(
            self.user_config
                .colors
                .get("activated_edges")
                .expect("You MUST supply an .active_edges key in your toml with a valid colour"),
        );
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

    fn draw_nodes(&self, painter: &egui::Painter) {
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

    fn clear_active_nodes(&mut self) {
        for node in self.passive_tree.nodes.values_mut() {
            node.active = false;
        }
        self.highlighted_path.clear();
        self.active_edges.clear();
        log::info!("Cleared all active nodes and paths.");
    }

    fn draw_rhs_menu(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("rhs_menu").show(ctx, |ui| {
            ui.heading("Menu");

            // Top Buttons Section
            self.draw_top_buttons(ui);

            // Search Functionality
            self.search(ui);

            // Start and Target Node Configuration
            self.pathing(ui);
        });
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

    fn pathing(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.heading("Node Configuration");

        ui.horizontal(|ui| {
            ui.label("Start Node:");
            ui.label(self.start_node_id.to_string());
        });

        ui.horizontal(|ui| {
            ui.label("Target Node:");
            let mut target_node_str = self.target_node_id.to_string();
            let response = ui.text_edit_singleline(&mut target_node_str);

            // Update only when focus is lost or Enter is pressed
            if response.lost_focus() && response.has_focus() {
                if let Ok(parsed) = target_node_str.parse::<usize>() {
                    if self
                        .passive_tree
                        .is_node_within_distance(self.start_node_id, parsed, 123)
                    {
                        self.target_node_id = parsed;
                        log::debug!("Target Node updated: {}", self.target_node_id);
                    } else {
                        log::warn!(
                            "Node {} is not within 123 steps of start node {}",
                            parsed,
                            self.start_node_id
                        );
                    }
                } else {
                    log::error!("Invalid input for target node: {}", target_node_str);
                }
            }
        });
    }
}

impl TreeVis<'_> {
    /// Draw the debug information bar
    fn draw_debug_bar(&self, ctx: &egui::Context) {
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
    fn get_debug_bar_contents(
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

    fn screen_to_world_x(&self, sx: f32) -> f64 {
        ((sx - 500.0) / self.zoom + self.camera.borrow().0) as f64
    }

    fn screen_to_world_y(&self, sy: f32) -> f64 {
        ((sy - 500.0) / self.zoom + self.camera.borrow().1) as f64
    }
}

impl TreeVis<'_> {
    pub fn check_and_activate_nodes(&mut self) {
        let active_nodes: HashSet<usize> = self
            .passive_tree
            .nodes
            .iter()
            .filter(|(_, node)| node.active)
            .map(|(id, _)| *id)
            .collect();

        let current_active_count = active_nodes.len();
        let previous_active_count = ACTIVE_NODE_COUNT.load(Ordering::Relaxed);

        if current_active_count <= previous_active_count {
            return;
        }

        ACTIVE_NODE_COUNT.store(current_active_count, Ordering::Relaxed);

        if active_nodes.len() < 2 {
            log::debug!("Not enough active nodes for pathfinding.");
            return;
        }

        let mut visited_nodes: HashSet<NodeId> = HashSet::new();
        let mut updated_nodes = false;

        active_nodes.iter().enumerate().try_for_each(|(i, &start)| {
            active_nodes.iter().skip(i + 1).try_for_each(|&end| {
                if visited_nodes.contains(&start) && visited_nodes.contains(&end) {
                    return ControlFlow::Continue::<()>(());
                }

                log::debug!("Attempting to find path between {} and {}", start, end);

                if !self.passive_tree.edges.iter().any(|edge| {
                    (edge.start == start && edge.end == end)
                        || (edge.start == end && edge.end == start)
                }) {
                    let start_time = Instant::now();
                    let path = self.passive_tree.find_path(start, end);

                    if !path.is_empty() {
                        log::debug!("Path found: {:?} between {} and {}", path, start, end);

                        path.iter().for_each(|&node_id| {
                            if let Some(node) = self.passive_tree.nodes.get_mut(&node_id) {
                                if !node.active {
                                    node.active = true;
                                    updated_nodes = true;
                                }
                                visited_nodes.insert(node_id);
                            }
                        });

                        let duration = start_time.elapsed();
                        log::debug!("Path activated in {:?}", duration);
                    } else {
                        log::debug!("No path found between {} and {}", start, end);
                    }
                }

                ControlFlow::Continue::<()>(())
            })
        });

        if updated_nodes {
            log::debug!("Nodes activated along paths.");
            self.active_nodes = active_nodes;
        } else {
            log::debug!("No new nodes activated.");
        }
    }
}
impl TreeVis<'_> {
    pub fn check_and_activate_edges(&mut self) {
        let mut visited_edges: HashSet<(NodeId, NodeId)> = HashSet::new();
        let active_nodes: Vec<_> = self
            .passive_tree
            .nodes
            .iter()
            .filter(|(_, node)| node.active)
            .map(|(id, _)| *id)
            .collect();

        // Don't recompute paths and edges unless we've increased the number of nodes
        let current_active_count = active_nodes.len();
        let previous_active_count = ACTIVE_NODE_COUNT.load(Ordering::Relaxed);
        if current_active_count <= previous_active_count {
            return;
        }

        active_nodes.iter().enumerate().try_for_each(|(i, &start)| {
            active_nodes.iter().skip(i + 1).try_for_each(|&end| {
                if visited_edges.contains(&(start, end)) || visited_edges.contains(&(end, start)) {
                    return ControlFlow::Continue::<()>(());
                }

                if self.passive_tree.edges.iter().any(|edge| {
                    (edge.start == start && edge.end == end)
                        || (edge.start == end && edge.end == start)
                }) {
                    log::debug!("Edge found and activated between {} and {}", start, end);

                    visited_edges.insert((start, end));
                }

                ControlFlow::Continue::<()>(())
            })
        });
        self.active_edges = visited_edges;

        log::debug!(
            "Edge activation completed. Active edges: {:?}",
            self.active_edges
        );
    }

    // Other related functions like draw_edges, check_and_activate_nodes, etc.
}
impl TreeVis<'_> {
    pub fn select_node(&mut self, node_id: usize) {
        if let Some(character) = &mut self.current_character {
            if !character.activated_node_ids.contains(&node_id) {
                log::info!("Selecting node: {}", node_id);
                self.target_node_id = node_id;
                self.process_path_to_active_node(node_id, true); // Activate the path
            }
        }
    }
    pub fn hover_node(&mut self, node_id: usize) {
        if let Some(node) = self.passive_tree.nodes.get(&node_id) {
            if !node.active {
                log::info!("Hovering over node: {}", node_id);

                // Highlight the shortest path to an active node (do not activate)
                if let Some(shortest_path) = self.process_path_to_active_node(node_id, false) {
                    // log::debug!("Highlighting path: {:?}", shortest_path);
                    self.highlighted_path = shortest_path;
                }
            }
        }
    }

    /// Processes the shortest path from a given node to the nearest active node.
    /// If `activate` is true, nodes and edges along the path will be activated.
    fn process_path_to_active_node(
        &mut self,
        node_id: usize,
        activate: bool,
    ) -> Option<Vec<usize>> {
        if let Some((shortest_path, active_node_id)) =
            self.find_shortest_path_to_active_node(node_id)
        {
            log::debug!(
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
            log::warn!("No active node found to connect to {}", node_id);
        }

        None
    }

    /// Finds the shortest path from the given node to the nearest active node.
    fn find_shortest_path_to_active_node(&self, target_node: usize) -> Option<(Vec<usize>, usize)> {
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
    fn update_active_edges(&mut self, path: Vec<usize>) {
        for window in path.windows(2) {
            if let [start, end] = window {
                self.active_edges.insert((*start, *end));
                self.active_edges.insert((*end, *start));
                log::debug!("Edge activated: ({}, {})", start, end);
            }
        }
    }

    const HOVER_RADIUS: f32 = 100.0;
    fn get_hovered_node(&self, ctx: &egui::Context) -> Option<usize> {
        // Logic to determine hovered node from mouse position
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

    fn get_target_node(&self) -> Option<usize> {
        // Logic to determine if a target node has been selected
        Some(self.target_node_id)
    }
}
