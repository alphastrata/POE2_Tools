#![allow(dead_code, unused_imports, unused_assignments, unused_variables)]

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use bevy::{prelude::*, time::common_conditions::on_timer, utils::HashSet};
use bevy_cosmic_edit::{
    cosmic_text::{Attrs, BufferRef, Edit, Family, Metrics},
    prelude::*,
    CosmicBackgroundColor, MaxLines, Placeholder,
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
                    cleanup_search_results,
                    scan_for_and_higlight_results,
                ),
            );

        app.add_systems(
            Update,
            (
                read_searchtext.run_if(on_timer(Duration::from_millis(32))),
                (mark_matches).after(read_searchtext).chain(),
            ),
        );
        log::debug!("SearchTools plugin is enabled");
    }
}

fn spawn_search_textbox(mut commands: Commands, mut font_system: ResMut<CosmicFontSystem>) {
    let attrs = Attrs::new()
        .family(Family::Name("Victor Mono"))
        .color(CosmicColor::rgb(255, 255, 255));

    let cosmic_edit = commands
        .spawn((
            TextEdit,
            CosmicEditBuffer::new(&mut font_system, Metrics::new(16., 20.)).with_rich_text(
                &mut font_system,
                vec![("", attrs)],
                attrs,
            ),
            Node {
                // position_type: PositionType::Absolute,
                // // display: Display::Flex,
                // // justify_content: JustifyContent::Center,
                // // align_items: AlignItems::Center,
                // margin: UiRect::all(Val::Auto),
                width: Val::Percent(25.0),
                height: Val::Percent(10.0),
                ..default()
            },
            CosmicBackgroundColor(Color::rgba(0.0, 0.0, 1.0, 0.0)),
            SearchMarker,
            MaxLines(1),
            Placeholder::new(
                "Start typing...",
                Attrs::new().color(Color::from(bevy::color::palettes::css::GRAY).to_cosmic()),
            ),
            Visibility::Hidden,
        ))
        .id();

    commands.insert_resource(FocusedWidget(Some(cosmic_edit)));
}

fn process_searchbox_visibility_toggle(
    windows: Query<&Window>,
    mut commands: Commands,
    mut searchbox_query: Query<(Entity, &mut Node), With<SearchMarker>>,
    mut searchbox_state: ResMut<SearchState>,
) {
    let Ok((sb, mut node)) = searchbox_query.get_single_mut() else {
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

    if let Some(cursor_pos) = windows.single().cursor_position() {
        log::debug!("Cursor Position: {:?}", cursor_pos);

        node.left = Val::Px(cursor_pos.x);
        node.top = Val::Px(cursor_pos.y);
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
            colour_events.send(NodeColourReq(ent, materials.node_base.clone()));
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
) {
    for (transform, _) in &search_results {
        gizmos.circle_2d(
            transform.translation().truncate(),
            80.0,
            Color::hsl(120.0, 1.0, 0.5), // any hue you like
        );
    }
}

fn cleanup_search_results(
    mut commands: Commands,
    mut searchbox_state: ResMut<SearchState>,
    query: Query<(Entity, &NodeMarker), With<SearchResult>>,
) {
    if !searchbox_state.open || &searchbox_state.search_query == "" {
        searchbox_state.search_query.clear();
        for (ent, _) in &query {
            commands.entity(ent).remove::<SearchResult>();
        }
    }
}
