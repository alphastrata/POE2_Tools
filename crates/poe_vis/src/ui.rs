#![allow(unused_mut)]
use crate::{
    background_services::tailwind_to_egui,
    components::{NodeActive, NodeMarker, UIGlyph},
    consts::TAILWIND_COLOURS_AS_STR,
    events::{
        ClearAllReqReq, ClearSearchResults, DrawCircleReq, LoadCharacterReq, MoveCameraReq,
        NodeActivationReq, NodeDeactivationReq, OptimiseReq, OverrideCharacterClassReq,
        OverrideCharacterNodesReq, SaveCharacterAsReq, SaveCharacterReq, SyncCharacterReq,
    },
    resources::{
        ActiveCharacter, CameraSettings, Optimiser, PathRepairRequired, SearchState, ToggleUi,
        UIOptimiserStatToggles,
    },
    PassiveTreeWrapper,
};
use bevy::{ecs::system::SystemParam, prelude::*, utils::HashMap};
use bevy_egui::{
    egui::{self, Align, Context, SidePanel, TextBuffer},
    EguiClipboard, EguiContext, EguiContexts, EguiPlugin,
};

use poe_tree::{
    character::CharacterClass,
    consts::get_char_starts_node_map,
    nodes::PoeNode,
    stats::{arithmetic::PlusPercentage, Stat},
    type_wrappings::NodeId,
    PassiveTree,
};

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
            // space
            .init_resource::<ActiveNodeCounter>()
            .init_resource::<ToggleUi>()
            // space
            .add_plugins(EguiPlugin)
            .add_systems(Update, update_active_nodecount)
            .add_systems(
                Update,
                (
                    ui_lockout_handler,
                    egui_ui_system.run_if(resource_exists_and_equals(ToggleUi(true))),
                ),
            );
    }
}
fn ui_lockout_handler(mut contexts: EguiContexts, mut settings: ResMut<CameraSettings>) {
    let ctx = contexts.ctx_mut();
    settings.egui_has_lock =
        ctx.wants_pointer_input() || ctx.wants_keyboard_input() || ctx.is_pointer_over_area()
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

/// you can only have a maximum of 16 args to a bevy system, so you need to wrap them.
#[derive(SystemParam)]
struct EguiResources<'w> {
    character: Res<'w, ActiveCharacter>,
    togglers: ResMut<'w, UIOptimiserStatToggles>,
    optimiser: Res<'w, Optimiser>,
    clipboard: ResMut<'w, EguiClipboard>,
    settings: Res<'w, CameraSettings>,
    tree: Res<'w, PassiveTreeWrapper>,
}
//TODO: Wrap egui events, and egui queries.

fn egui_ui_system(
    active_nodes: Query<&NodeMarker, With<NodeActive>>,
    camera_query: Query<&mut OrthographicProjection, With<Camera2d>>,
    mut load_tx: EventWriter<LoadCharacterReq>,
    mut clear_all_tx: EventWriter<ClearAllReqReq>,
    mut clear_search_results_tx: EventWriter<ClearSearchResults>,
    mut contexts: EguiContexts,
    mut draw_circle: EventWriter<DrawCircleReq>,
    mut change_class_tx: EventWriter<OverrideCharacterClassReq>,
    mut move_camera_tx: EventWriter<MoveCameraReq>,
    mut optimiser_req: EventWriter<OptimiseReq>,
    mut save_as_tx: EventWriter<SaveCharacterAsReq>,
    mut save_tx: EventWriter<SaveCharacterReq>,
    mut override_path_tx: EventWriter<OverrideCharacterNodesReq>,
    sys_params: EguiResources,
) {
    //you can only have a maximum of 16 args to a bevy system, so you need to wrap them.
    let EguiResources {
        character,
        togglers,
        optimiser,
        clipboard,
        settings,
        tree,
    } = sys_params;

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
        character,
        optimiser,
        settings,
        tree,
        change_class_tx,
        clear_all_tx,
        clear_search_results_tx,
        clipboard,
        draw_circle,
        move_camera_tx,
        override_path_tx,
        togglers,
        optimiser_req,
    );
}

fn rhs_menu(
    contexts: &mut EguiContexts,
    camera_query: Query<&mut OrthographicProjection, With<Camera2d>>,
    active_nodes: Query<&NodeMarker, With<NodeActive>>,
    character: Res<ActiveCharacter>,
    optimiser: Res<Optimiser>,
    settings: Res<CameraSettings>,
    tree: Res<PassiveTreeWrapper>,

    mut change_class_tx: EventWriter<OverrideCharacterClassReq>,
    mut clear_all_tx: EventWriter<ClearAllReqReq>,
    mut clear_search_results_tx: EventWriter<ClearSearchResults>,
    mut clipboard: ResMut<EguiClipboard>,
    mut draw_circle: EventWriter<DrawCircleReq>,
    mut move_camera_tx: EventWriter<MoveCameraReq>,
    mut override_path_tx: EventWriter<OverrideCharacterNodesReq>,
    mut togglers: ResMut<UIOptimiserStatToggles>,
    mut optimiser_req: EventWriter<OptimiseReq>,
) -> egui::InnerResponse<()> {
    let ctx = contexts.ctx_mut();

    let projection = camera_query.single(); // Get current zoom level

    const UI_HOVER_BASE_RADIUS: f32 = 20.0;
    SidePanel::right("rhs").resizable(true).show(ctx, |ui| {
        ui.heading("Active Nodes");
        ui.separator();

        let mut selected_class = character.character_class;
        egui::ComboBox::from_label("CLASS:")
            .selected_text(selected_class.as_str())
            .show_ui(ui, |ui| {
                let mut keys: Vec<&&str> = get_char_starts_node_map().keys().collect();
                keys.sort();
                for class_str in keys.iter() {
                    let class = CharacterClass::from_str(class_str);
                    if ui
                        .selectable_value(&mut selected_class, class, class.as_str())
                        .clicked()
                    {
                        change_class_tx.send(OverrideCharacterClassReq(selected_class));
                        return;
                    }
                }
            });

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
                                .on_hover_text(format!("{root_stats:?}"));

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
                                    .on_hover_text(format!("{stats:?}"));

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
        character.activated_node_ids.iter().for_each(|nm| {
            if let Some(poe_node) = tree.nodes.get(nm) {
                for stat in poe_node.as_passive_skill(&tree).stats() {
                    all_stats.push((*nm, stat));
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

        let points_spent = character.activated_node_ids.len();
        ui.heading(format!("{points_spent}/123 passives allocated"));
        ui.separator();

        if ui.button("Copy path to clipboard").clicked() {
            let path: Vec<_> = active_nodes.iter().map(|nm| **nm).collect();
            let path_str = format!("{path:?}");
            clipboard.set_text(&path_str);
        }

        ui.separator();
        if ui.button("Clear All").clicked() {
            clear_all_tx.send(ClearAllReqReq);
            //NOTE: It's important to bail if we run a clear all so none of the other egui shitfuckery has a chance to override values causing our ever-listening bevy systems to accidentally reactivate shit.
            return;
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
            togglers,
            // clear_all_tx,
            override_path_tx,
        );
    })
}

fn topbar_menu_system(
    contexts: &mut EguiContexts<'_, '_>,
    save_tx: &mut EventWriter<SaveCharacterReq>,
    save_as_tx: &mut EventWriter<SaveCharacterAsReq>,
    load_tx: &mut EventWriter<LoadCharacterReq>,
    clear_all_tx: &mut EventWriter<ClearAllReqReq>,
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
                        clear_all_tx.send(ClearAllReqReq);
                        log::trace!("Selected file: {path:?}");
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
    let mut groups: HashMap<String, (f32, usize, Vec<NodeId>)> =
        stats.fold(HashMap::new(), |mut acc, (node_id, stat)| {
            let name = stat.name();
            let value = stat.value();
            acc.entry(name.to_string())
                .and_modify(|(sum, count, node_ids)| {
                    *sum += value;
                    *count += 1;
                    node_ids.push(node_id);
                })
                .or_insert((value, 1, vec![node_id]));
            acc
        });

    let mut entries: Vec<_> = groups.into_iter().collect();
    entries.sort_by(|(_, (sum_a, _, _)), (_, (sum_b, _, _))| {
        sum_b
            .partial_cmp(sum_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    entries
        .into_iter()
        .for_each(|(name, (sum, count, node_ids))| {
            let response = ui.label(format!("{name}: {sum} ({count} nodes)"));
            if response.hovered() {
                let scaled_radius = (20.0 * projection_scale).abs();
                node_ids.iter().for_each(|&node_id| {
                    if let Some(node) = tree.nodes.get(&node_id) {
                        let origin = Vec3::new(node.wx, -node.wy, 0.0);
                        draw_circle.send(DrawCircleReq {
                            radius: scaled_radius,
                            origin,
                            mat: "amber-500".into(),
                            glyph: UIGlyph::from_millis(500),
                        });
                    }
                });
            }
        });
}

fn draw_optimiser_ui(
    ui: &mut egui::Ui,
    tree: &Res<PassiveTreeWrapper>,
    mut character: Res<ActiveCharacter>,
    optimiser: Res<Optimiser>,
    mut optimiser_req: EventWriter<OptimiseReq>,
    mut draw_circle: EventWriter<DrawCircleReq>,
    _projection: &OrthographicProjection,
    mut togglers: ResMut<UIOptimiserStatToggles>,
    // mut clear_all_tx: EventWriter<ClearAllReqReq>,
    mut override_path_tx: EventWriter<OverrideCharacterNodesReq>,
) {
    ui.heading("Optimiser");
    ui.separator();

    // Aggregate stats from root + active nodes
    let mut aggregator: Vec<(NodeId, &Stat)> = Vec::new();

    if let Some(root) = tree.nodes.get(&character.starting_node) {
        aggregator.extend(
            root.as_passive_skill(tree)
                .stats()
                .iter()
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
                        .iter()
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
                    aggregator
                        .iter()
                        .filter(|(_, s)| s.name() == stat_name)
                        .filter_map(|(id, _)| tree.nodes.get(id))
                        .for_each(|node| {
                            draw_circle.send(DrawCircleReq {
                                radius: 90.0,
                                origin: Vec3::new(node.wx, -node.wy, 10.0),
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
        //FIXME: This range needs to actually be the (current level ... 123-current level), or 1, 123 if no points are spent.
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
    let optimise_disabled = toggled_stat.is_none() || !optimiser.is_available();
    let button = ui.add_enabled(!optimise_disabled, egui::Button::new("Optimise"));

    if button.clicked() && !optimise_disabled {
        if let Some(stat_name) = toggled_stat.clone() {
            optimiser_req.send(OptimiseReq {
                selector: Box::new(move |s: &Stat| s.name() == stat_name), // Now using owned String
                delta: togglers.delta,
            });
        }
    } else {
        // Display paths if available
        optimiser
            .results
            .iter()
            .enumerate()
            .for_each(|(idx, path)| {
                let Some(stat_name) = toggled_stat.as_ref() else {
                    log::error!("Unable to pull path for stat? something is horribly wrong!");
                    return;
                };

                let s = |s: &Stat| s.name().to_owned() == *stat_name;

                let mut stat_map: HashMap<String, f32> = HashMap::new();
                path.iter()
                    .flat_map(|nid| tree.nodes.get(nid))
                    .flat_map(|pnode| {
                        let skill = pnode.as_passive_skill(tree);
                        skill.stats().iter().filter(|stat_item| s(stat_item))
                    })
                    .for_each(|stat_item| {
                        *stat_map
                            .entry(stat_item.as_str().to_string())
                            .or_insert(0.0) += stat_item.value();
                    });

                let stat_totals = stat_map
                    .iter()
                    .map(|(k, v)| format!("{k}: {v}"))
                    .collect::<Vec<_>>()
                    .join(", ");

                let button = ui.button(format!(
                    "Path #{} for {} (Totals: {})",
                    idx + 1,
                    stat_name,
                    stat_totals
                ));

                //TODO: we need to show with red circles the path nodes they'd have to UNSPEND!
                // Hover preview
                if button.hovered() {
                    path.iter()
                        // Don't show them the nodes they already have.
                        .filter(|nid| !character.activated_node_ids.contains(nid))
                        .filter_map(|id| tree.nodes.get(id))
                        .for_each(|node| {
                            draw_circle.send(DrawCircleReq {
                                radius: 95.0,
                                // have to go above the nodes to see the circles, and slightly larger than them.
                                origin: Vec3::new(node.wx, -node.wy, 10.0),
                                mat: "rose-700".into(),
                                glyph: UIGlyph::from_millis(125),
                            });
                        });
                }

                // Click handling
                if button.clicked() {
                    log::debug!("Replacing existing path with new one...");
                    // clear_all_tx.send(ClearAllReqReq); // Override handles it's own clearing to make fewer changes (and because the ordering of bevy systems is a pain)
                    override_path_tx.send(OverrideCharacterNodesReq(path.clone()));
                }
            });
    }
    //TODO: more optimiser controls...
}
