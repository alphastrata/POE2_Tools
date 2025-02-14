use std::borrow::BorrowMut;

use crate::{
    background_services::tailwind_to_egui,
    components::{NodeActive, NodeMarker, UIGlyph},
    consts::TAILWIND_COLOURS_AS_STR,
    events::{
        ClearAll, ClearSearchResults, DrawCircleReq, LoadCharacterReq, MoveCameraReq,
        NodeDeactivationReq, SaveCharacterAsReq, SaveCharacterReq,
    },
    resources::{ActiveCharacter, CameraSettings, SearchState},
    PassiveTreeWrapper,
};
use bevy::{prelude::*, utils::HashMap};
use bevy_egui::{
    egui::{self, Align, Context, SidePanel},
    EguiClipboard, EguiContext, EguiContexts, EguiPlugin,
};

use poe_tree::{
    nodes::PoeNode,
    stats::{arithmetic::PlusPercentage, Stat},
    type_wrappings::NodeId,
    PassiveTree,
};

pub struct UIPlugin;

#[derive(Resource, Default)]
struct UICapturesInput(pub bool);

#[derive(Deref, DerefMut, Resource, Default)]
struct Toggles(HashMap<String, bool>);

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
            // space
            .init_resource::<UICapturesInput>()
            .init_resource::<ActiveNodeCounter>() // store node count
            .init_resource::<Toggles>()
            // space
            .add_plugins(EguiPlugin)
            .add_systems(Update, update_active_nodecount) // track how many are active
            .add_systems(Update, egui_ui_system); // draw EGUI
    }
}

#[derive(Resource, Default)]
struct ActiveNodeCounter(pub usize);

// just count the active nodes
fn update_active_nodecount(
    active_nodes: Query<&NodeMarker, With<NodeActive>>,
    mut counter: Local<ActiveNodeCounter>,
) {
    counter.0 = active_nodes.iter().count();
}
fn egui_ui_system(
    mut contexts: EguiContexts,

    camera_query: Query<&mut OrthographicProjection, With<Camera2d>>,
    move_camera_tx: EventWriter<MoveCameraReq>,

    active_nodes: Query<&NodeMarker, With<NodeActive>>,
    mut clear_all_tx: EventWriter<ClearAll>,
    clear_search_results_tx: EventWriter<ClearSearchResults>,

    mut save_tx: EventWriter<SaveCharacterReq>,
    mut save_as_tx: EventWriter<SaveCharacterAsReq>,
    mut load_tx: EventWriter<LoadCharacterReq>,
    draw_circle: EventWriter<DrawCircleReq>,

    tree: Res<PassiveTreeWrapper>,
    character: ResMut<ActiveCharacter>,
    settings: Res<CameraSettings>,
    searchbox_state: ResMut<SearchState>,
    clipboard: ResMut<EguiClipboard>,

    toggles: Local<Toggles>,
) {
    topbar_menu_system(
        &mut contexts,
        &mut save_tx,
        &mut save_as_tx,
        &mut load_tx,
        &mut clear_all_tx,
    );

    rhs_menu(
        &mut contexts,
        camera_query,
        active_nodes,
        clear_all_tx,
        clear_search_results_tx,
        move_camera_tx,
        tree,
        character,
        settings,
        draw_circle,
        // searchbox_state,
        clipboard,
        toggles,
    );
}

fn rhs_menu(
    contexts: &mut EguiContexts,
    camera_query: Query<&mut OrthographicProjection, With<Camera2d>>,
    active_nodes: Query<&NodeMarker, With<NodeActive>>,
    mut clear_all_tx: EventWriter<ClearAll>,
    mut clear_search_results_tx: EventWriter<ClearSearchResults>,
    mut move_camera_tx: EventWriter<MoveCameraReq>,
    tree: Res<PassiveTreeWrapper>,
    mut character: ResMut<ActiveCharacter>,
    settings: Res<CameraSettings>,
    mut draw_circle: EventWriter<DrawCircleReq>,
    // searchbox_state: ResMut<SearchState>,
    mut clipboard: ResMut<EguiClipboard>,
    toggles: Local<Toggles>,
) -> egui::InnerResponse<()> {
    let ctx = contexts.ctx_mut();

    let projection = camera_query.single(); // Get current zoom level

    const UI_HOVER_BASE_RADIUS: f32 = 20.0;
    SidePanel::right("rhs").resizable(true).show(ctx, |ui| {
        ui.heading("Active Nodes");
        ui.separator();

        ui.heading(format!("CLASS: {}", character.character_class));
        ui.separator();

        ui.collapsing("Node details...", |ui| {
            ui.set_min_height(300.0); // Ensure enough space
            egui::ScrollArea::vertical().show(ui, |ui| {
                // (Existing individual node display code remains here …)
                if let Some(root) = tree.nodes.get(&character.starting_node) {
                    let root_stats = root.as_passive_skill(&tree).stats();
                    ui.horizontal(|ui| {
                        fmt_for_ui(root, &tree, ui);
                        ui.with_layout(egui::Layout::right_to_left(Align::RIGHT), |ui| {
                            let button = ui
                                .small_button("🏠")
                                .on_hover_text(format!("{:?}", root_stats));

                            if button.clicked() {
                                move_camera_tx.send(MoveCameraReq(Vec3::new(
                                    root.wx,
                                    -root.wy,
                                    settings.min_zoom,
                                )));
                                log::trace!("Move2Node triggered...");
                            }

                            if button.hovered() {
                                let scaled_radius = (UI_HOVER_BASE_RADIUS * projection.scale).abs();
                                let origin = Vec3::new(root.wx, -root.wy, 0.0);
                                draw_circle.send(DrawCircleReq {
                                    radius: scaled_radius,
                                    origin,
                                    mat: "pink-500".into(),
                                    glyph: UIGlyph::from_millis(500),
                                });
                            }
                        });
                    });
                }

                let active_nodes: Vec<&NodeMarker> = active_nodes
                    .into_iter()
                    .filter(|nm| nm.0 != character.starting_node)
                    .collect();

                active_nodes.iter().for_each(|nm| {
                    if let Some(poe_node) = tree.nodes.get(&nm.0) {
                        let stats = poe_node.as_passive_skill(&tree).stats();
                        ui.horizontal(|ui| {
                            fmt_for_ui(poe_node, &tree, ui);
                            ui.with_layout(egui::Layout::right_to_left(Align::RIGHT), |ui| {
                                let button = ui
                                    .small_button(format!("{}", nm.0))
                                    .on_hover_text(format!("{:?}", stats));

                                if button.clicked() {
                                    move_camera_tx.send(MoveCameraReq(Vec3::new(
                                        poe_node.wx,
                                        -poe_node.wy,
                                        settings.min_zoom,
                                    )));
                                    log::trace!("Move2Node triggered...");
                                }

                                if button.hovered() {
                                    let scaled_radius =
                                        (UI_HOVER_BASE_RADIUS * projection.scale).abs();
                                    let origin = Vec3::new(poe_node.wx, -poe_node.wy, 0.0);
                                    draw_circle.send(DrawCircleReq {
                                        radius: scaled_radius,
                                        origin,
                                        mat: "pink-500".into(),
                                        glyph: UIGlyph::from_millis(500),
                                    });
                                }
                            });
                        });
                    }
                });
            });
        });
        // In rhs_menu, build aggregated stats with (NodeId, &Stat) instead of using search_query.
        let mut all_stats: Vec<(NodeId, &Stat)> = Vec::new();
        if let Some(root) = tree.nodes.get(&character.starting_node) {
            for stat in root.as_passive_skill(&tree).stats() {
                all_stats.push((character.starting_node, stat));
            }
        }
        active_nodes.iter().for_each(|nm| {
            if let Some(poe_node) = tree.nodes.get(&nm.0) {
                for stat in poe_node.as_passive_skill(&tree).stats() {
                    all_stats.push((nm.0, stat));
                }
            }
        });
        ui.collapsing("Aggregated Stats", |ui| {
            display_aggregated_stats(
                ui,
                all_stats.into_iter(),
                &tree,
                &mut draw_circle,
                projection.scale,
            );
        });
        ui.separator();

        ui.heading(format!("{} Points Spent", active_nodes.iter().len()));
        ui.separator();

        if ui.button("Copy path to clipboard").clicked() {
            let path: Vec<_> = active_nodes.iter().map(|nm| **nm).collect();
            let path_str = format!("{:?}", path); // yields [31765, 722, ...]
            clipboard.set_contents(&path_str);
        }

        ui.separator();
        if ui.button("Clear All").clicked() {
            clear_all_tx.send(ClearAll);
        }

        ui.separator();
        if ui.button("Clear Search Results").clicked() {
            clear_search_results_tx.send(ClearSearchResults);
        }

        draw_optimiser_ui(ui, &tree, character, draw_circle, projection, toggles);
    })
}

/// Adjust signature/args/resources as needed (e.g. need active_nodes query for aggregator, node_deactivation_tx etc.).
/// Example usage of tailwind_to_egui, TAILWIND_COLOURS_AS_STR, plus toggles & par_take_while for new path:
fn draw_optimiser_ui(
    ui: &mut egui::Ui,
    tree: &Res<PassiveTreeWrapper>,
    mut character: ResMut<ActiveCharacter>,
    mut draw_circle: EventWriter<DrawCircleReq>,
    projection: &OrthographicProjection,
    mut toggles: Local<Toggles>,
    // e.g. for swapping active nodes:
    // mut node_deactivation_tx: EventWriter<NodeDeactivationReq>,
    // mut node_activation_tx: EventWriter<NodeActivationReq>,
    // for generating egui colors:
    // tailwind_to_egui_fn: fn(&str) -> egui::Color32,
) {
    ui.heading("Optimiser");
    ui.separator();

    // Build aggregated stats from root + active nodes, grouped by name
    let mut aggregator: Vec<(NodeId, &Stat)> = Vec::new();
    if let Some(root) = tree.nodes.get(&character.starting_node) {
        for stat in root.as_passive_skill(tree).stats() {
            aggregator.push((character.starting_node, stat));
        }
    }
    character.activated_node_ids.iter().for_each(|nm| {
        if *nm != character.starting_node {
            if let Some(node) = tree.nodes.get(&nm) {
                for stat in node.as_passive_skill(tree).stats() {
                    aggregator.push((*nm, stat));
                }
            }
        }
    });
    // Distinct stat names
    let mut distinct_stats: Vec<String> = aggregator
        .iter()
        .map(|(_, s)| s.name().to_string())
        .collect();
    distinct_stats.sort();
    distinct_stats.dedup();

    ui.label(format!(
        "Root: {}",
        tree.nodes[&character.starting_node].name
    ));
    ui.separator();

    let palette = TAILWIND_COLOURS_AS_STR;
    for (i, stat_name) in distinct_stats.iter().enumerate() {
        // pick color
        let color_str = palette[i % palette.len()];
        let egui_col = tailwind_to_egui(color_str);

        // current toggle state
        let mut is_toggled = toggles.0.get(stat_name).copied().unwrap_or(false);

        ui.horizontal(|ui| {
            ui.label(" "); // just a spacer
            let response = ui.colored_label(egui_col, stat_name);
            ui.checkbox(&mut is_toggled, "");

            // if updated, store it
            if Some(&is_toggled) != toggles.0.get(stat_name) {
                toggles.0.insert(stat_name.clone(), is_toggled);
            }

            if response.hovered() {
                let radius = 20.0 * projection.scale;
                aggregator
                    .iter()
                    .filter(|(_, s)| s.name() == stat_name)
                    .for_each(|(node_id, _)| {
                        if let Some(node) = tree.nodes.get(node_id) {
                            let origin = Vec3::new(node.wx, -node.wy, 0.0);
                            draw_circle.send(DrawCircleReq {
                                radius,
                                origin,
                                mat: color_str.into(),
                                glyph: UIGlyph::from_millis(500),
                            });
                        }
                    });
            }
        });
    }
    ui.separator();

    // For each toggled stat, create a list of candidate paths
    for stat_name in &distinct_stats {
        let Some(true) = toggles.get(stat_name) else {
            return;
        };

        let candidate = Stat::from_key_value(stat_name, 0.0);
        // NOTE: why no good way to discard the float... hmmm, and match on our stricter types..
        let selector = move |s: &Stat| s.name() == candidate.name();

        let candidate_paths = tree.take_while(
            character.starting_node,
            selector,
            character.activated_node_ids.len(),
        );

        candidate_paths
            .iter()
            .enumerate()
            // Create buttons and track interaction states
            .map(|(idx, path)| {
                let label = format!("Path #{} for {}", idx + 1, stat_name);
                let button = ui.button(label);
                (idx, path, button)
            })
            // Process paths with hovered buttons
            .filter(|(_, _, button)| button.hovered())
            // Draw preview for hovered paths
            .for_each(|(_, path, button)| {
                // Draw path preview
                path.iter()
                    .filter_map(|node_id| tree.nodes.get(node_id))
                    .for_each(|node| {
                        draw_circle.send(DrawCircleReq {
                            radius: 20.0,
                            origin: Vec3::new(node.wx, -node.wy, 0.0),
                            mat: "sky-400".into(),
                            glyph: UIGlyph::from_millis(25),
                        });
                    });

                // Handle click events
                if button.clicked() {
                    println!("Clicked");
                    character.activated_node_ids.clear();
                    character.activated_node_ids.extend(path.iter().copied());
                }
            });
    }

    ui.separator();
    ui.label("Optimiser controls go here...");
}

fn topbar_menu_system(
    contexts: &mut EguiContexts<'_, '_>,
    save_tx: &mut EventWriter<SaveCharacterReq>,
    save_as_tx: &mut EventWriter<SaveCharacterAsReq>,
    load_tx: &mut EventWriter<LoadCharacterReq>,
    clear_all_tx: &mut EventWriter<ClearAll>,
) {
    let ctx = contexts.ctx_mut();
    egui::TopBottomPanel::top("top_menu").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui
                    .button("Import")
                    .on_hover_text("import a file from Path of Building")
                    .clicked()
                {
                    log::trace!("Import selected...");
                    if let Some(path) = rfd::FileDialog::new()
                        .set_title("Import character")
                        .add_filter("PoB File", &["xml", "toml"])
                        .pick_file()
                    {
                        clear_all_tx.send(ClearAll);
                        log::trace!("Selected file: {:?}", path);
                        load_tx.send(LoadCharacterReq(path));
                        return;
                    }
                }

                if ui
                    .button("Export")
                    .on_hover_text("export a file for import into Path of Building")
                    .clicked()
                {
                    // TODO: Implement Export logic here.
                }
                if ui
                    .button("Save")
                    .on_hover_text("saves your current build")
                    .clicked()
                {
                    save_tx.send(SaveCharacterReq);
                }
                if ui
                    .button("Save As")
                    .on_hover_text("save your current build somewhere")
                    .clicked()
                {
                    if let Some(path) = rfd::FileDialog::new().set_title("Save as...").save_file() {
                        save_as_tx.send(SaveCharacterAsReq(path));
                    }
                }
                if ui.button("Exit").on_hover_text("quits poe_vis").clicked() {
                    // TODO: Implement Exit logic here.
                }
            });
        });
    });
}

fn fmt_for_ui(poe_node: &PoeNode, tree: &PassiveTree, ui: &mut egui::Ui) {
    let as_passive = poe_node.as_passive_skill(tree);
    if as_passive.is_notable() {
        ui.label(&poe_node.name);
    } else {
        let name = as_passive.name();
        let color = if name.to_lowercase().contains("dexterity") {
            egui::Color32::GREEN
        } else if name.to_lowercase().contains("strength") {
            egui::Color32::RED
        } else if name.to_lowercase().contains("intelligence") {
            egui::Color32::BLUE
        } else {
            egui::Color32::WHITE
        };
        //TODO: If it was a passive attribute we need to offer three buttons + for each..
        // When the user clicks those buttons, we need to update stats...
        ui.colored_label(color, name);
    }
}

fn display_aggregated_stats<'t>(
    ui: &mut egui::Ui,
    stats: impl Iterator<Item = (NodeId, &'t Stat)>,
    tree: &PassiveTreeWrapper,
    draw_circle: &mut EventWriter<DrawCircleReq>,
    projection_scale: f32,
) {
    let mut groups: HashMap<String, (f32, usize, Vec<NodeId>)> = HashMap::new();
    for (node_id, stat) in stats {
        let name = stat.name();
        let value = stat.value();
        groups
            .entry(name.to_string())
            .and_modify(|(sum, count, node_ids)| {
                *sum += value;
                *count += 1;
                node_ids.push(node_id);
            })
            .or_insert((value, 1, vec![node_id]));
    }
    let mut entries: Vec<_> = groups.into_iter().collect();
    entries.sort_by(|(a, _), (b, _)| a.cmp(b));
    for (name, (sum, count, node_ids)) in entries {
        let response = ui.label(format!("{}: {} ({} nodes)", name, sum, count));
        if response.hovered() {
            let scaled_radius = (20.0 * projection_scale).abs();
            for node_id in node_ids {
                if let Some(node) = tree.nodes.get(&node_id) {
                    let origin = Vec3::new(node.wx, -node.wy, 0.0);
                    draw_circle.send(DrawCircleReq {
                        radius: scaled_radius,
                        origin,
                        mat: "amber-500".into(),
                        glyph: UIGlyph::from_millis(500),
                    });
                }
            }
        }
    }
}
