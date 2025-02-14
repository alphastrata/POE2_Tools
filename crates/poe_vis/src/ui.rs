use crate::{
    components::{NodeActive, NodeMarker, UIGlyph},
    events::{
        ClearAll, ClearSearchResults, DrawCircleReq, LoadCharacterReq, MoveCameraReq,
        NodeDeactivationReq, SaveCharacterAsReq, SaveCharacterReq,
    },
    resources::{ActiveCharacter, CameraSettings, SearchState},
    PassiveTreeWrapper,
};
use bevy::{prelude::*, utils::HashMap};
use bevy_egui::{
    egui::{self, Align, SidePanel},
    EguiClipboard, EguiContexts, EguiPlugin,
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

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
            // space
            .init_resource::<UICapturesInput>()
            .init_resource::<ActiveNodeCounter>() // store node count
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
    mut camera_query: Query<&mut OrthographicProjection, With<Camera2d>>,
    active_nodes: Query<&NodeMarker, With<NodeActive>>,
    mut clear_all_tx: EventWriter<ClearAll>,
    clear_search_results_tx: EventWriter<ClearSearchResults>,

    move_camera_tx: EventWriter<MoveCameraReq>,
    mut save_tx: EventWriter<SaveCharacterReq>,
    mut save_as_tx: EventWriter<SaveCharacterAsReq>,
    mut load_tx: EventWriter<LoadCharacterReq>,
    draw_circle: EventWriter<DrawCircleReq>,

    tree: Res<PassiveTreeWrapper>,
    character: Res<ActiveCharacter>,
    mut contexts: EguiContexts,
    settings: Res<CameraSettings>,
    searchbox_state: ResMut<SearchState>,
    clipboard: ResMut<EguiClipboard>,
) {
    topbar_menu_system(
        &mut contexts,
        &mut save_tx,
        &mut save_as_tx,
        &mut load_tx,
        &mut clear_all_tx,
    );

    rhs_menu(
        camera_query,
        active_nodes,
        clear_all_tx,
        clear_search_results_tx,
        move_camera_tx,
        tree,
        character,
        &mut contexts,
        settings,
        draw_circle,
        searchbox_state,
        clipboard,
    );
}

fn rhs_menu(
    mut camera_query: Query<&mut OrthographicProjection, With<Camera2d>>,
    active_nodes: Query<&NodeMarker, With<NodeActive>>,
    mut clear_all_tx: EventWriter<ClearAll>,
    mut clear_search_results_tx: EventWriter<ClearSearchResults>,
    mut move_camera_tx: EventWriter<MoveCameraReq>,
    tree: Res<PassiveTreeWrapper>,
    character: Res<ActiveCharacter>,
    contexts: &mut EguiContexts,
    settings: Res<CameraSettings>,
    mut draw_circle: EventWriter<DrawCircleReq>,
    searchbox_state: ResMut<SearchState>,
    mut clipboard: ResMut<EguiClipboard>,
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
    })
}

// Refactor topbar_menu_system to accept an immutable reference to egui::Context.
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
    let mut groups: HashMap<String, (f32, usize, NodeId)> = HashMap::new();
    for (node_id, stat) in stats {
        let name = stat.name();
        let value = stat.value();
        groups
            .entry(name.to_string())
            .and_modify(|(sum, count, _)| {
                *sum += value;
                *count += 1;
            })
            .or_insert((value, 1, node_id));
    }
    let mut entries: Vec<_> = groups.into_iter().collect();
    entries.sort_by(|(a, _), (b, _)| a.cmp(b));
    for (name, (sum, count, rep_node_id)) in entries {
        let response = ui.label(format!("{}: {} ({} nodes)", name, sum, count));
        if response.hovered() {
            if let Some(node) = tree.nodes.get(&rep_node_id) {
                let scaled_radius = (20.0 * projection_scale).abs();
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
