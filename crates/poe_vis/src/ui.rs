use crate::{
    components::{NodeActive, NodeMarker},
    events::{ClearAll, LoadCharacterReq, MoveCameraReq, NodeDeactivationReq, SaveCharacterReq},
    resources::{ActiveCharacter, CameraSettings, SearchState},
    PassiveTreeWrapper,
};
use bevy::{prelude::*, utils::HashMap};
use bevy_egui::{
    egui::{self, Align, SidePanel},
    EguiContexts, EguiPlugin,
};

use poe_tree::{
    nodes::PoeNode,
    stats::{arithmetic::PlusPercentage, Stat},
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
    active_nodes: Query<&NodeMarker, With<NodeActive>>,
    mut clear_all_tx: EventWriter<ClearAll>,
    move_camera_tx: EventWriter<MoveCameraReq>,
    mut save_tx: EventWriter<SaveCharacterReq>,
    mut load_tx: EventWriter<LoadCharacterReq>,
    tree: Res<PassiveTreeWrapper>,
    character: Res<ActiveCharacter>,
    mut contexts: EguiContexts,
    settings: Res<CameraSettings>,
    searchbox_state: ResMut<SearchState>,
) {
    topbar_menu_system(&mut contexts, &mut save_tx, &mut load_tx, &mut clear_all_tx);

    rhs_menu(
        active_nodes,
        clear_all_tx,
        move_camera_tx,
        tree,
        character,
        &mut contexts,
        settings,
        searchbox_state,
    );
}

fn rhs_menu(
    active_nodes: Query<'_, '_, &NodeMarker, With<NodeActive>>,
    mut clear_all_tx: EventWriter<'_, ClearAll>,
    mut move_camera_tx: EventWriter<'_, MoveCameraReq>,
    tree: Res<'_, PassiveTreeWrapper>,
    character: Res<'_, ActiveCharacter>,
    contexts: &mut EguiContexts<'_, '_>,
    settings: Res<'_, CameraSettings>,
    searchbox_state: ResMut<SearchState>,
) -> egui::InnerResponse<()> {
    let ctx = contexts.ctx_mut();
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
                            if ui
                                .small_button("🏠")
                                .on_hover_text(format!("{:?}", root_stats))
                                .clicked()
                            {
                                move_camera_tx.send(MoveCameraReq(Vec3::new(
                                    root.wx,
                                    -root.wy,
                                    settings.min_zoom,
                                )));
                                log::trace!("Move2Node triggered...");
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
                                if ui
                                    .small_button(format!("{}", nm.0))
                                    .on_hover_text(format!("{:?}", stats))
                                    .clicked()
                                {
                                    move_camera_tx.send(MoveCameraReq(Vec3::new(
                                        poe_node.wx,
                                        -poe_node.wy,
                                        settings.min_zoom,
                                    )));
                                    log::trace!("Move2Node triggered...");
                                }
                            });
                        });
                    }
                });
            });
        });

        // Now, aggregate stats from all nodes (root + active) and display a summary.
        let mut all_stats: Vec<&Stat> = Vec::new();
        if let Some(root) = tree.nodes.get(&character.starting_node) {
            all_stats.extend(root.as_passive_skill(&tree).stats());
        }

        active_nodes.iter().for_each(|nm| {
            if let Some(poe_node) = tree.nodes.get(&nm.0) {
                all_stats.extend(poe_node.as_passive_skill(&tree).stats());
            }
        });
        ui.collapsing("Aggregated Stats", |ui| {
            // TODO: We are FAR from displaying all that can be displayed here!
            display_aggregated_stats(ui, all_stats.into_iter(), searchbox_state);
        });
        ui.separator();

        ui.heading(format!("{} Points Spent", active_nodes.iter().len()));
        ui.separator();

        if ui.button("Clear All").clicked() {
            clear_all_tx.send(ClearAll);
        }
    })
}

// Refactor topbar_menu_system to accept an immutable reference to egui::Context.
fn topbar_menu_system(
    contexts: &mut EguiContexts<'_, '_>,
    save_tx: &mut EventWriter<SaveCharacterReq>,
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
                    // TODO: Implement Save As logic here.
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

/// Aggregates stats that are of PlusPercentage/Plus/MinusPercentage etc type into a HashMap where the key is the stat name
/// and the value is a tuple of (total value, count of nodes).
fn aggregate_stats<'t>(stats: impl Iterator<Item = &'t Stat>) -> HashMap<String, (f32, usize)> {
    let mut groups: HashMap<String, (f32, usize)> = HashMap::new();
    stats.for_each(|stat| {
        let (name, value) = (stat.as_str(), stat.value());

        groups
            .entry(name.to_owned())
            .and_modify(|(sum, count)| {
                *sum += value;
                *count += 1;
            })
            .or_insert((value, 1));
    });
    groups
}

/// In your UI system, call this function to display aggregated stat summaries.
fn display_aggregated_stats<'t>(
    ui: &mut egui::Ui,
    stats: impl Iterator<Item = &'t Stat>,
    mut searchbox_state: ResMut<SearchState>,
) {
    let groups = aggregate_stats(stats);
    //sort alphabetically
    let mut entries: Vec<_> = groups.into_iter().collect();
    entries.sort_by(|(name_a, _), (name_b, _)| name_a.cmp(name_b));
    entries.into_iter().for_each(|(name, (sum, count))| {
        let response = ui.label(format!("{}: {} ({} nodes)", name, sum, count));
        if response.hovered() {
            // NOTE: bastardise the search system to get circling going on...
            searchbox_state.search_query = name;
        }
    });
}
