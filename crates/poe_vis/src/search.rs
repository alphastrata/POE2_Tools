#![allow(dead_code, unused_imports, unused_assignments, unused_variables)]

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use bevy::{prelude::*, time::common_conditions::on_timer, utils::HashSet};
use bevy_cosmic_edit::{
    cosmic_text::{Attrs, BufferRef, Edit, Family, Metrics},
    // placeholder::Placeholder,
    prelude::*,
    MaxLines,
    Placeholder,
};
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
            .add_systems(Startup, spawn_search_textbox)
            .add_systems(
                Update,
                (
                    process_searchbox_visibility_toggle.run_if(on_event::<ShowSearch>),
                    scan_for_search_results,
                ),
            );

        app.add_systems(
            Update,
            (
                read_searchtext.run_if(on_timer(Duration::from_millis(32))),
                (mark_matches, cleanup_search_results)
                    .after(read_searchtext)
                    .chain(),
            ),
        );
        log::debug!("SearchTools plugin is enabled");
    }
}

fn spawn_search_textbox(mut commands: Commands, mut font_system: ResMut<CosmicFontSystem>) {
    let attrs = Attrs::new()
        .family(Family::Name("Victor Mono"))
        .color(CosmicColor::rgb(0x94, 0x00, 0xD3));

    let cosmic_edit = commands
        .spawn((
            TextEdit,
            CosmicEditBuffer::new(&mut font_system, Metrics::new(16., 20.)).with_rich_text(
                &mut font_system,
                vec![("", attrs)],
                attrs,
            ),
            Node {
                width: Val::Percent(25.),
                height: Val::Percent(8.),
                ..default()
            },
            SearchMarker,
            MaxLines(1),
            Placeholder::new(
                "Start searching...",
                Attrs::new().color(Color::from(bevy::color::palettes::css::GRAY).to_cosmic()),
            ),
            Visibility::Hidden,
        ))
        .id();

    commands.insert_resource(FocusedWidget(Some(cosmic_edit)));
}

// Search:
fn process_searchbox_visibility_toggle(
    mut commands: Commands,
    mut searchbox_query: Query<Entity, With<SearchMarker>>,
    mut searchbox_state: ResMut<SearchState>,
) {
    let Ok(sb) = searchbox_query.get_single_mut() else {
        log::warn!("Unable to get searchbox...");
        return;
    };

    searchbox_state.open = !searchbox_state.open;
    match searchbox_state.open {
        true => {
            commands.entity(sb).remove::<Visibility>();
            commands.entity(sb).insert(Visibility::Visible);
        }
        false => {
            commands.entity(sb).remove::<Visibility>();
            commands.entity(sb).insert(Visibility::Hidden);
        }
    }
}

fn read_searchtext(
    mut searchbox_state: ResMut<SearchState>,
    query: Query<&CosmicEditor, With<SearchMarker>>,
) {
    query.iter().for_each(|buffer| {
        if let BufferRef::Owned(buffer) = buffer.editor.buffer_ref() {
            buffer.lines.iter().for_each(|l| {
                let mut txt = l.clone().into_text();

                if searchbox_state.search_query != txt {
                    txt = txt.trim_start_matches("/").to_string();
                    std::mem::swap(&mut searchbox_state.search_query, &mut txt);
                }
            });
        }
    });
}

fn mark_matches(
    tree: Res<PassiveTreeWrapper>,
    searchbox_state: Res<SearchState>,
    commands: Commands,
    query: Query<(Entity, &NodeMarker)>,
) {
    if searchbox_state.search_query.len() >= SEARCH_THRESHOLD {
        let add_me: HashSet<NodeId> = tree
            .fuzzy_search_nodes(&searchbox_state.search_query)
            .into_iter()
            .collect();

        let l_cmd = Arc::new(Mutex::new(commands));
        query.par_iter().for_each(|(ent, nm)| {
            if add_me.contains(&(**nm)) {
                match l_cmd.lock() {
                    Ok(mut cmd) => {
                        cmd.entity(ent).insert(SearchResult);
                        log::debug!("SearchResult {}", **nm);
                    }
                    Err(e) => {
                        log::error!("{}", e);
                    }
                }
            }
        });
    }
}

fn scan_for_search_results(
    mut colour_events: EventWriter<NodeColourReq>,
    search_results: Query<(Entity, &NodeMarker), With<SearchResult>>,

    game_materials: Res<GameMaterials>,
) {
    search_results.into_iter().for_each(|(ent, _nm)| {
        colour_events.send(NodeColourReq(ent, game_materials.purple.clone()));
    });
}

fn cleanup_search_results(
    mut commands: Commands,
    mut searchbox_state: ResMut<SearchState>,
    query: Query<(Entity, &NodeMarker), With<SearchResult>>,
) {
    // Cleanup if closed OR if the searchbox is cleared (i.e ctrl+a + delete)
    if !searchbox_state.open || searchbox_state.search_query.is_empty() {
        log::debug!("SearchResult cleanup begins...");
        searchbox_state.search_query.clear();

        query.iter().for_each(|(ent, nm)| {
            commands.entity(ent).remove::<SearchResult>();
            log::trace!("Removing highlight from {}", nm.0);
        });
    }
}
