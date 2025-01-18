//$ src\visualiser.rs
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    default::Default,
    ops::ControlFlow,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::Instant,
};

use egui::accesskit::Tree;
use log::debug;

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

    /// Draw the debug information bar
    fn draw_debug_bar(&self, ctx: &egui::Context) {
        let (mouse_info, zoom_info, hovered_node_info) = self.get_debug_bar_contents(ctx);

        egui::TopBottomPanel::bottom("debug_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(mouse_info);
                ui.separator();
                ui.label(zoom_info);
                ui.separator();
                ui.label(hovered_node_info);
            });
        });
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
    path: Vec<usize>,
    // for multi-step pathing
    path_nodes: Vec<usize>,

    /// Store edges of the current path
    // NOTE: mostly used for drawing.
    active_edges: HashSet<(usize, usize)>,
    active_nodes: HashSet<usize>,

    // Config-driven colours
    color_map: HashMap<String, String>,

    current_character: Option<UserCharacter>,
    last_save_time: std::time::Instant,

    user_config: UserConfig,

    /// Mapped controls from self.user_config
    controls: HashMap<String, egui::Key>,
}

// Helper Functions
impl<'p> TreeVis<'p> {
    // Camera consts
    const SCALE_FACTOR: f32 = 1.0;
    const ZOOM_MIN: f32 = 0.0; // Minimum zoom level
    const ZOOM_MAX: f32 = 1.0; // Maximum zoom level
    const ZOOM_STEP: f32 = 0.0001; // Step size for zoom changes

    // Node size constants
    const BASE_RADIUS: f32 = 8.0;
    const NOTABLE_MULTIPLIER: f32 = 1.5; // Scale notable nodes
    const NAMELESS_MULTIPLIER: f32 = 1.0; // Scale nameless nodes

    fn update_fuzzy_search(&mut self, ctx: &egui::Context) {
        if self.is_fuzzy_search_open() {
            egui::Window::new("Fuzzy Search")
                .collapsible(true)
                .show(ctx, |ui| {
                    let response = ui.text_edit_singleline(&mut self.search_query);
                    if response.changed() {
                        self.search_results =
                            self.passive_tree.fuzzy_search_nodes(&self.search_query);
                    }
                    egui::CollapsingHeader::new("Results").show(ui, |ui| {
                        for &id in &self.search_results {
                            let node = &self.passive_tree.nodes[&id];
                            if ui.selectable_label(false, &node.name).double_clicked() {
                                self.go_to_node(id);
                            }
                        }
                    });
                });
        }
    }

    fn update_hover(&mut self, mx: f32, my: f32) {
        self.hovered_node = self
            .passive_tree
            .nodes
            .iter()
            .find(|(_, node)| {
                let dx = mx - node.wx as f32;
                let dy = my - node.wy as f32;
                (dx * dx + dy * dy).sqrt() < 10.0 // Hover threshold
            })
            .map(|(id, _)| *id);
    }

    fn auto_fit(&mut self) {
        let (min_x, max_x) = self
            .passive_tree
            .nodes
            .values()
            .map(|node| node.wx)
            .fold((f64::MAX, f64::MIN), |(min, max), x| {
                (min.min(x), max.max(x))
            });

        let (min_y, max_y) = self
            .passive_tree
            .nodes
            .values()
            .map(|node| node.wy)
            .fold((f64::MAX, f64::MIN), |(min, max), y| {
                (min.min(y), max.max(y))
            });

        let width = max_x - min_x;
        let height = max_y - min_y;

        self.zoom = (1.0 / width as f32).min(1.0 / height as f32) * 0.9; // Adjust zoom for screen size
        self.camera = RefCell::new((
            (min_x + max_x) as f32 / 2.0, // Center camera
            (min_y + max_y) as f32 / 2.0,
        ));
    }

    fn current_zoom_level(&self) -> f32 {
        self.zoom
    }

    fn enable_fuzzy_search(&self) {
        self.fuzzy_search_open.store(true, Ordering::Relaxed);
    }

    fn disable_fuzzy_search(&self) {
        self.fuzzy_search_open.store(false, Ordering::Relaxed);
    }

    fn is_fuzzy_search_open(&self) -> bool {
        self.fuzzy_search_open.load(Ordering::Relaxed)
    }

    fn camera_x(&self) -> f32 {
        self.camera.borrow().0
    }

    fn camera_y(&self) -> f32 {
        self.camera.borrow().1
    }

    fn camera_xy(&self) -> (f32, f32) {
        *self.camera.borrow()
    }

    fn find_arbitrary_path(&mut self) {
        if self.path_nodes.len() < 2 {
            return; // Need at least two nodes
        }

        let mut full_path = Vec::new();
        for pair in self.path_nodes.windows(2) {
            if let [start, target] = pair {
                let segment = self.passive_tree.find_shortest_path(*start, *target);
                if full_path.is_empty() {
                    full_path.extend(segment);
                } else {
                    full_path.extend(segment.iter().skip(1));
                }
            }
        }
        self.path = full_path;
    }

    fn find_path(&mut self, start: usize, target: usize) {
        let path = self.passive_tree.find_shortest_path(start, target);

        // Update the active path and edges.
        self.path = path.clone();
        self.active_edges.clear();
        for window in path.windows(2) {
            if let [a, b] = window {
                self.active_edges.insert((*a, *b));
                self.active_edges.insert((*b, *a));
            }
        }
    }

    pub fn new(
        passive_tree: &'p mut PassiveTree,
        user_config: UserConfig,
        current_character: Option<UserCharacter>,
    ) -> Self {
        // Initialize controls from the user configuration
        // let controls = user_config
        //     .controls
        //     .iter()
        //     .map(|(action, key)| (action.clone(), *key))
        //     .collect();

        // vis.initialize_camera_and_zoom();
        Self {
            camera: RefCell::new((700.0, 700.0)),
            zoom: 0.07,
            passive_tree,
            hovered_node: None, // No node hovered initially

            // Fuzzy-search-related
            fuzzy_search_open: AtomicBool::new(false), // Search not open initially
            search_query: String::new(),               // Empty search query
            search_results: Vec::new(),                // No search results initially

            // Path-finder-related
            start_node_id: 0,             // Default to the root or initial node
            target_node_id: 0,            // Default to no target node
            path: Vec::new(),             // No path initially
            active_edges: HashSet::new(), // No edges highlighted initially
            active_nodes: HashSet::new(),
            // Config-driven colours
            color_map: HashMap::new(),

            // Multi-step pathing
            path_nodes: Vec::new(), // No intermediate nodes

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
        }
    }
    fn go_to_node(&self, id: usize) {
        self.move_camera_to_node(id);
        self.disable_fuzzy_search();
    }
    fn select_node(&mut self, node_id: usize) {
        if let Some(character) = &mut self.current_character {
            if !character.activated_node_ids.contains(&node_id) {
                character.activated_node_ids.push(node_id);
                self.save_character();
            }
        }
    }

    fn save_character(&mut self) {
        if let Some(character) = &self.current_character {
            character.save_to_toml("data/last_character.toml");
            self.last_save_time = std::time::Instant::now();
        }
    }

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

    fn highlight_activated_nodes(&self, painter: &egui::Painter) {
        if let Some(character) = &self.current_character {
            for &node_id in &character.activated_node_ids {
                if let Some(node) = self.passive_tree.nodes.get(&node_id) {
                    painter.circle_filled(
                        egui::pos2(
                            self.world_to_screen_x(node.wx),
                            self.world_to_screen_y(node.wy),
                        ),
                        10.0, // slightly larger size
                        parse_color(
                            self.color_map
                                .get("green")
                                .unwrap_or(&"#29D398".to_string()),
                        ),
                    );
                }
            }
        }
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

    /// Precompute debug bar contents to avoid borrow conflicts
    fn get_debug_bar_contents(&self, ctx: &egui::Context) -> (String, String, String) {
        // Get mouse position
        let mouse_pos = ctx.input(|input| input.pointer.hover_pos().unwrap_or_default());
        let mouse_info = format!("Mouse: ({:.2}, {:.2})", mouse_pos.x, mouse_pos.y);

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

        (mouse_info, zoom_info, hovered_node_info)
    }

    fn initialize_camera_and_zoom(&mut self) {
        let (min_x, max_x) = self
            .passive_tree
            .nodes
            .values()
            .map(|node| node.wx)
            .fold((f64::MAX, f64::MIN), |(min, max), x| {
                (min.min(x), max.max(x))
            });

        let (min_y, max_y) = self
            .passive_tree
            .nodes
            .values()
            .map(|node| node.wy)
            .fold((f64::MAX, f64::MIN), |(min, max), y| {
                (min.min(y), max.max(y))
            });

        let width = max_x - min_x;
        let height = max_y - min_y;

        self.zoom = (1.0 / width as f32).min(1.0 / height as f32) * 800.0;
        *self.camera.borrow_mut() = ((min_x + max_x) as f32 / 2.0, (min_y + max_y) as f32 / 2.0);
    }

    fn world_to_screen_x(&self, wx: f64) -> f32 {
        (wx as f32 - self.camera.borrow().0) * self.zoom + 500.0
    }

    fn world_to_screen_y(&self, wy: f64) -> f32 {
        (wy as f32 - self.camera.borrow().1) * self.zoom + 500.0
    }
}

impl TreeVis<'_> {
    fn click_node(&mut self, node_id: NodeId) {
        if let Some(node) = self.passive_tree.nodes.get_mut(&node_id) {
            node.active = !node.active;
            if node.active {
                if !self.path.contains(&node_id) {
                    self.path.push(node_id);
                }
            } else {
                self.path.retain(|&id| id != node_id);
            }
            log::debug!("Node {} active state toggled to: {}", node_id, node.active);
            log::debug!("Current path nodes: {:?}", self.path);
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

        log::debug!("Edge activation completed.");
    }

    pub fn redraw_tree(&self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();

            self.draw_edges(&painter);
            self.draw_nodes(&painter);
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
}

impl TreeVis<'_> {
    fn clear_active_nodes(&mut self) {
        for node in self.passive_tree.nodes.values_mut() {
            node.active = false;
        }
        self.path.clear();
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
