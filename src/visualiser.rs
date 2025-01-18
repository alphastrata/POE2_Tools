//$ src\visualiser.rs
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    default::Default,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::config::{parse_color, UserCharacter};
use crate::{config::UserConfig, data::poe_tree::PassiveTree};

pub struct TreeVis<'p> {
    camera: RefCell<(f32, f32)>,
    zoom: f32,
    passive_tree: &'p PassiveTree,
    hovered_node: Option<usize>,

    // Fuzzy-search-related
    fuzzy_search_open: AtomicBool,
    search_query: String,
    search_results: Vec<usize>,

    // Path-finder-related
    start_node_id: usize,
    target_node_id: usize,
    path: Vec<usize>,

    /// Store edges of the current path
    active_edges: HashSet<(usize, usize)>,

    // Config-driven colours
    color_map: HashMap<String, String>,

    // for multi-step pathing
    path_nodes: Vec<usize>,

    current_character: Option<UserCharacter>,
    last_save_time: std::time::Instant,

    user_config: UserConfig,

    /// Mapped controls from self.user_config
    controls: HashMap<String, egui::Key>,
}

impl<'p> eframe::App for TreeVis<'p> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // IO
        self.handle_mouse(ctx);
        self.update_active_edges();

        //DEBUG:
        self.draw_debug_bar(ctx);

        // drawing
        self.redraw_tree(ctx);

        //TODO: draw rhs menu

        //todo: draw top menu (open tree, char etc..)
    }
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

    fn redraw_tree(&self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();
            let zoom = 1.0 + self.zoom; // Zoom level for scaling nodes

            // Draw edges
            for edge in &self.passive_tree.edges {
                if let (Some(source), Some(target)) = (
                    self.passive_tree.nodes.get(&edge.from),
                    self.passive_tree.nodes.get(&edge.to),
                ) {
                    let sx = self.world_to_screen_x(source.wx);
                    let sy = self.world_to_screen_y(source.wy);
                    let tx = self.world_to_screen_x(target.wx);
                    let ty = self.world_to_screen_y(target.wy);

                    painter.line_segment(
                        [egui::pos2(sx, sy), egui::pos2(tx, ty)],
                        egui::Stroke::new(1.0, egui::Color32::GRAY),
                    );
                }
            }

            // Draw nodes
            self.passive_tree.nodes.values().for_each(|node| {
                let sx = self.world_to_screen_x(node.wx);
                let sy = self.world_to_screen_y(node.wy);

                let mut radius = Self::BASE_RADIUS / zoom;

                if node.is_notable {
                    radius *= Self::NOTABLE_MULTIPLIER;
                }

                if !node.name.chars().any(|c| c.is_digit(10)) {
                    radius *= Self::NAMELESS_MULTIPLIER;
                }

                //TODO: get from config
                let color = node.base_color(&self.user_config);

                painter.circle_filled(egui::pos2(sx, sy), radius, color);
            });
        });
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
        self.update_active_edges();
    }

    fn update_active_edges(&mut self) {
        self.active_edges.clear();
        for window in self.path.windows(2) {
            if let [a, b] = window {
                self.active_edges.insert((*a, *b));
                self.active_edges.insert((*b, *a));
            }
        }
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
        passive_tree: &'p PassiveTree,
        user_config: UserConfig,
        current_character: Option<UserCharacter>,
    ) -> Self {
        // Initialize controls from the user configuration
        // let controls = user_config
        //     .controls
        //     .iter()
        //     .map(|(action, key)| (action.clone(), *key))
        //     .collect();

        let mut vis = Self {
            camera: RefCell::new((0.0, 0.0)), // Start camera at origin
            zoom: 0.02,
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

            // Config-driven colours
            color_map: HashMap::new(),

            // Multi-step pathing
            path_nodes: Vec::new(), // No intermediate nodes

            current_character,
            last_save_time: std::time::Instant::now(), // Set to the current time

            user_config,
            controls: HashMap::new(),
        };

        vis.initialize_camera_and_zoom();
        vis
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
        ((wx as f32 - self.camera.borrow().0) * self.zoom + 500.0) as f32
    }

    fn world_to_screen_y(&self, wy: f64) -> f32 {
        ((wy as f32 - self.camera.borrow().1) * self.zoom + 500.0) as f32
    }
}
