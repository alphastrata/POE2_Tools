#![allow(dead_code, unused_imports, unused_assignments, unused_variables)]

use bevy::{prelude::*, utils::HashSet};
use bevy_cosmic_edit::{
    cosmic_text::{Attrs, Family, Metrics},
    // placeholder::Placeholder,
    prelude::*,
    MaxLines,
    Placeholder,
};

use crate::{
    components::{NodeMarker, SearchMarker, Skill},
    events::NodeColourReq,
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
            .add_systems(Update, handle_searched_nodes);

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

fn handle_searched_nodes(
    tree: Res<PassiveTreeWrapper>,
    query: Query<(&NodeMarker, &Skill)>,
    search: ResMut<SearchState>,
    highlighter: EventWriter<NodeColourReq>,
    materials: Res<GameMaterials>,
) {
    // if the poenode tree.nodes.get(NodeMarker.0).unwrap().name() or skill.name .lower() contains any of the search text..
    // send the nodeId to the NodeColourReq(node_id, matierals.purple)

    // send base_node colour reqs to all items in the search_results any modification of the search_query.
}

fn highlight_search_results(search: ResMut<SearchState>, highlighter: EventWriter<NodeColourReq>) {

    // use the search.results to regex out NodeIds (u32s)
    // send highlight requests to all of them.
}
