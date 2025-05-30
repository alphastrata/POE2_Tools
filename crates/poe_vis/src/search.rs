#![allow(dead_code, unused_imports, unused_assignments, unused_variables)]

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use bevy::{
    color::{self, palettes::tailwind},
    prelude::*,
    time::common_conditions::on_timer,
    utils::HashSet,
};
use bevy_cosmic_edit::{
    cosmic_text::{Attrs, BufferRef, Edit, Family, Metrics},
    prelude::*,
    CosmicBackgroundColor, MaxLines, Placeholder,
};
use bevy_egui::{egui, EguiContexts};
use poe_tree::type_wrappings::NodeId;

use crate::{
    components::{NodeMarker, SearchMarker, SearchResult, Skill, UIGlyph},
    consts::{self, DEFAULT_SEARCH_HIGHLIGHT_DURATION, SEARCH_THRESHOLD},
    events::{ClearSearchResults, DrawCircleReq, NodeColourReq, ShowSearch},
    materials::{self, GameMaterials},
    resources::{CameraSettings, SearchState},
    PassiveTreeWrapper,
};

pub struct SearchToolsPlugin;

impl Plugin for SearchToolsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SearchState {
            search_query: String::new(),
            open: false,
        });

        let font_bytes: &[u8] = include_bytes!("../assets/fonts/VictorMono-Regular.ttf");
        let font_config = CosmicFontConfig {
            fonts_dir_path: None,
            font_bytes: Some(vec![font_bytes]),
            load_system_fonts: true,
        };

        app.add_plugins(CosmicEditPlugin { font_config })
            .add_systems(
                Update,
                (
                    process_searchbox_visibility_toggle.run_if(on_event::<ShowSearch>),
                    cleanup_search_results.run_if(on_event::<ClearSearchResults>),
                    (scan_for_and_highlight_results, mark_matches)
                        .run_if(SearchState::should_search),
                    egui_searchbox_system.run_if(SearchState::is_open),
                ),
            );

        log::debug!("SearchTools plugin is enabled");
    }
}

fn process_searchbox_visibility_toggle(mut search_state: ResMut<SearchState>) {
    search_state.open = !search_state.open;
    log::trace!("Searchbox open = {}", &search_state.open);
}

// simple egui window with a text field
fn egui_searchbox_system(mut search_state: ResMut<SearchState>, mut contexts: EguiContexts) {
    egui::Window::new("Search").show(contexts.ctx_mut(), |ui| {
        let field =
            egui::TextEdit::singleline(&mut search_state.search_query).hint_text("start typing...");
        ui.add(field).request_focus();
    });
}

fn mark_matches(
    tree: Res<PassiveTreeWrapper>,
    searchbox_state: Res<SearchState>,
    mut commands: Commands,
    current_highlight: Query<(Entity, &NodeMarker), With<SearchResult>>,
    mut colour_events: EventWriter<NodeColourReq>,
    all_nodes: Query<(Entity, &NodeMarker)>,
    materials: Res<GameMaterials>,
) {
    if searchbox_state.search_query.len() < SEARCH_THRESHOLD {
        return;
    }
    let new_matches: HashSet<NodeId> = tree
        .fuzzy_search_nodes_and_skills(&searchbox_state.search_query)
        .into_iter()
        .collect();

    current_highlight
        .iter()
        .filter(|(_, nm)| !new_matches.contains(&nm.0))
        .for_each(|(ent, _)| {
            commands.entity(ent).remove::<SearchResult>();
            colour_events.send(NodeColourReq(ent, materials.node_base.clone_weak()));
        });

    let cmd = Arc::new(Mutex::new(commands));
    all_nodes.par_iter().for_each(|(ent, nm)| {
        if new_matches.contains(&nm.0) {
            if let Ok(mut c) = cmd.lock() {
                c.entity(ent).insert(SearchResult);
            }
        }
    });
}

fn scan_for_and_highlight_results(
    mut draw_requests: EventWriter<DrawCircleReq>,
    search_results: Query<(&GlobalTransform, &NodeMarker), With<SearchResult>>,
) {
    search_results.iter().for_each(|(tf, _)| {
        let origin = tf.translation().truncate().extend(0.0);
        draw_requests.send(DrawCircleReq {
            radius: 88.0,
            origin,
            mat: "orange-500".into(),
            glyph: UIGlyph::new_with_duration(DEFAULT_SEARCH_HIGHLIGHT_DURATION),
        });
    });
}

fn cleanup_search_results(
    mut commands: Commands,
    mut searchbox_state: ResMut<SearchState>,
    query: Query<(Entity, &NodeMarker), With<SearchResult>>,
) {
    if !searchbox_state.open || searchbox_state.search_query.is_empty() {
        searchbox_state.search_query.clear();
        query.iter().for_each(|(ent, _)| {
            commands.entity(ent).remove::<SearchResult>();
        });
    }
}
