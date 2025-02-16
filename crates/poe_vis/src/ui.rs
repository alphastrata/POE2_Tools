use std::borrow::BorrowMut;

use crate::{
    background_services::tailwind_to_egui,
    components::{NodeActive, NodeMarker, UIGlyph},
    consts::TAILWIND_COLOURS_AS_STR,
    events::{
        ClearAll, ClearSearchResults, DrawCircleReq, LoadCharacterReq, MoveCameraReq,
        NodeDeactivationReq, OptimiseReq, SaveCharacterAsReq, SaveCharacterReq,
    },
    resources::{ActiveCharacter, CameraSettings, Optimiser, SearchState},
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

#[derive(Resource, Default)]
struct Toggles {
    selections: HashMap<String, bool>,
    delta: usize,
}

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
    clipboard: ResMut<EguiClipboard>,

    toggles: Local<Toggles>,
    optimiser_req: EventWriter<OptimiseReq>,
    optimiser: ResMut<Optimiser>,
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
        optimiser,
        optimiser_req,
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
    character: ResMut<ActiveCharacter>,
    settings: Res<CameraSettings>,
    mut draw_circle: EventWriter<DrawCircleReq>,
    mut clipboard: ResMut<EguiClipboard>,
    optimiser: ResMut<Optimiser>,
    mut optimiser_req: EventWriter<OptimiseReq>,
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

        draw_optimiser_ui(
            ui,
            &tree,
            character,
            optimiser,
            optimiser_req,
            draw_circle,
            projection,
            toggles,
        );
    })
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

fn draw_optimiser_ui(
    ui: &mut egui::Ui,
    tree: &Res<PassiveTreeWrapper>,
    mut character: ResMut<ActiveCharacter>,
    optimiser: ResMut<Optimiser>,
    mut optimiser_req: EventWriter<OptimiseReq>,
    mut draw_circle: EventWriter<DrawCircleReq>,
    projection: &OrthographicProjection,
    mut togglers: Local<Toggles>,
) {
    ui.heading("Optimiser");
    ui.separator();

    // Aggregate stats from root + active nodes
    let mut aggregator: Vec<(NodeId, &Stat)> = Vec::new();

    if let Some(root) = tree.nodes.get(&character.starting_node) {
        aggregator.extend(
            root.as_passive_skill(tree)
                .stats()
                .into_iter()
                .map(|stat| (character.starting_node, stat)),
        );
    }

    character
        .activated_node_ids
        .iter()
        .filter(|&&id| id != character.starting_node)
        .for_each(|id| {
            if let Some(node) = tree.nodes.get(id) {
                aggregator.extend(
                    node.as_passive_skill(tree)
                        .stats()
                        .into_iter()
                        .map(|stat| (*id, stat)),
                );
            }
        });

    // Get distinct sorted stats
    let mut distinct_stats: Vec<String> = aggregator
        .iter()
        .map(|(_, s)| s.name().to_string())
        .collect();
    distinct_stats.sort();
    distinct_stats.dedup();

    // Display root node
    // TODO: we can remove this concept entirely really...
    ui.label(format!("Root: {}", character.starting_node));
    ui.separator();

    // Stat Selection UI
    let palette = TAILWIND_COLOURS_AS_STR;
    distinct_stats
        .iter()
        .enumerate()
        .for_each(|(i, stat_name)| {
            let color_str = palette[i % palette.len()];
            let egui_col = tailwind_to_egui(color_str);
            let mut is_toggled = *togglers.selections.get(stat_name).unwrap_or(&false);

            ui.horizontal(|ui| {
                ui.label(" ");
                let response = ui.colored_label(egui_col, stat_name);

                // Right-aligned checkbox
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.checkbox(&mut is_toggled, "").changed() {
                        // Ensure only one stat is toggled at a time
                        togglers.selections.iter_mut().for_each(|(_, v)| *v = false);
                        togglers.selections.insert(stat_name.clone(), is_toggled);
                    }
                });

                // Hover preview
                if response.hovered() {
                    let radius = 20.0 * projection.scale;
                    aggregator
                        .iter()
                        .filter(|(_, s)| s.name() == stat_name)
                        .filter_map(|(id, _)| tree.nodes.get(id))
                        .for_each(|node| {
                            draw_circle.send(DrawCircleReq {
                                radius,
                                origin: Vec3::new(node.wx, -node.wy, 0.0),
                                mat: color_str.into(),
                                glyph: UIGlyph::from_millis(500),
                            });
                        });
                }
            });
        });

    ui.separator();

    // Delta slider
    ui.horizontal(|ui| {
        ui.label("Delta:");
        ui.add(egui::Slider::new(&mut togglers.delta, 1..=10).text("Path length adjustment"));
    });

    // Get selected stat (if any)
    let toggled_stat = togglers.selections.iter().find_map(|(stat, &is_selected)| {
        if is_selected {
            Some(stat.clone())
        } else {
            None
        }
    });

    // Optimise button (only enabled if a stat is selected)
    let optimise_disabled = toggled_stat.is_none() || !optimiser.available();
    let button = ui.add_enabled(!optimise_disabled, egui::Button::new("Optimise"));

    if button.clicked() {
        if let Some(ref stat_name) = toggled_stat.clone() {
            let selector = move |s: &Stat| s.name() == stat_name.as_str();

            optimiser_req.send(OptimiseReq {
                selector: Box::new(selector), // Ensure it's boxed for dynamic dispatch
                delta: togglers.delta,
            });
        }
    }

    ui.separator();
    ui.label("Optimiser controls go here...");

    // Display paths if available
    optimiser
        .results
        .iter()
        .enumerate()
        .for_each(|(idx, path)| {
            let Some(stat) = toggled_stat.clone() else {
                log::error!("Unable to pull path for stat? something is horribly wrong!");
                return;
            };
            let button = ui.button(format!("Path #{} for {}", idx + 1, stat));

            // Hover preview
            if button.hovered() {
                path.iter()
                    .filter_map(|id| tree.nodes.get(id))
                    .for_each(|node| {
                        draw_circle.send(DrawCircleReq {
                            radius: 20.0,
                            origin: Vec3::new(node.wx, -node.wy, 0.0),
                            mat: "sky-400".into(),
                            glyph: UIGlyph::from_millis(25),
                        });
                    });
            }

            // Click handling
            if button.clicked() {
                character.activated_node_ids.clear();
                character.activated_node_ids.extend(path.iter().copied());
            }
        });

    ui.separator();
    ui.label("Optimiser controls go here...");
}
