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
    components::{NodeMarker, SearchMarker, SearchResult, Skill},
    consts::SEARCH_THRESHOLD,
    events::{NodeColourReq, ShowSearch},
    materials::{self, GameMaterials},
    resources::SearchState,
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
                    cleanup_search_results,
                    scan_for_and_higlight_results,
                    egui_searchbox_system,
                    mark_matches,
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
    if search_state.open {
        egui::Window::new("Search").show(contexts.ctx_mut(), |ui| {
            let field = egui::TextEdit::singleline(&mut search_state.search_query)
                .hint_text("start typing...");
            ui.add(field).request_focus();
        });
    }
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

fn scan_for_and_higlight_results(
    mut gizmos: Gizmos,
    search_results: Query<(&GlobalTransform, &NodeMarker), With<SearchResult>>,
    materials: Res<GameMaterials>,
) {
    search_results.iter().for_each(|(transform, _)| {
        gizmos.circle_2d(
            transform.translation().truncate(),
            80.0,
            tailwind::ORANGE_500,
        );
    });
}

fn cleanup_search_results(
    mut commands: Commands,
    mut searchbox_state: ResMut<SearchState>,
    query: Query<(Entity, &NodeMarker), With<SearchResult>>,
) {
    if !searchbox_state.open || searchbox_state.search_query.is_empty() {
        searchbox_state.search_query.clear();
        for (ent, _) in &query {
            commands.entity(ent).remove::<SearchResult>();
        }
    }
}
