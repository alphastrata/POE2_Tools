use bevy::prelude::*;
use poe_tree::{character, type_wrappings::NodeId};
use std::sync::{Arc, Mutex};

use crate::{
    components::*,
    resources::{ActiveCharacter, VirtualPath},
    PassiveTreeWrapper,
};

pub struct OverlaysAndPopupsPlugin;
impl Plugin for OverlaysAndPopupsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(VirtualPath::default());

        app.add_systems(Update, show_node_info)
            .add_systems(Startup, spawn_hover_text);
    }
}

fn show_node_info(
    windows: Query<&Window>,
    hovered: Query<(&Hovered, &NodeMarker, Option<&NodeActive>)>,
    mut hover_text_query: Query<(&mut Node, &mut Text), With<NodeHoverText>>,
    tree: Res<PassiveTreeWrapper>,
) {
    // Attempt to get the hover text's Node and Text components
    let Ok((mut node, mut text)) = hover_text_query.get_single_mut() else {
        log::warn!("Found no UI node to update...");
        return;
    };

    // Clear the text initially
    text.0.clear();

    // Check if there's hovered node information
    let mut found_info: Option<String> = None;
    for (_hovered, marker, _maybe_active) in hovered.iter() {
        if let Some(node_info) = tree.tree.nodes.get(&marker.0) {
            found_info = Some(format!("Node {}:\n{}", node_info.node_id, node_info.name));
            break;
        }
    }

    // Update the text content if we found information
    if let Some(info) = found_info {
        text.0 = info;
    }

    // Update the node's position in screen space
    if let Some(cursor_pos) = windows.single().cursor_position() {
        node.left = Val::Px(cursor_pos.x);
        node.top = Val::Px(cursor_pos.y);
    }
}

fn spawn_hover_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        // The text component
        Text::new(""),
        // Font configuration
        TextFont {
            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size: 22.0,
            ..default()
        },
        // Layout configuration for UI
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0), // Default starting position, whenever this is actually populated with content it'll be overridden.
            top: Val::Px(0.0),  // Default starting position
            ..default()
        },
        NodeHoverText, // Your custom marker
    ));
}

fn scan_for_hovered(
    mut commands: Commands,
    mut virt_path: ResMut<VirtualPath>,
    hovered: Query<(Entity, &NodeMarker), With<Hovered>>,
    edges: Query<(Entity, &EdgeMarker), Without<EdgeActive>>,

    character: Res<ActiveCharacter>,
    tree: Res<PassiveTreeWrapper>,
) {
    let targets: Vec<NodeId> = character.activated_node_ids.iter().copied().collect();

    hovered.iter().for_each(|(ent, nm)| {
        virt_path.nodes = tree.shortest_to_target_from_any_of(**nm, &targets);
        commands.entity(ent).insert(VirtualPathMember);
    });
    virt_path.nodes.sort();

    // Take only the ref to the vp so we can use fewer mutexes.
    let m_virtpath = Arc::new(&virt_path);

    let m_cmd = Arc::new(Mutex::new(&mut commands));
    let mut scratch = vec![];
    let m_virt_edges = Arc::new(Mutex::new(&mut scratch));

    edges.par_iter().for_each(|(ent, edg)| {
        if m_virtpath.contains_edge(edg) {
            m_virt_edges.lock().unwrap().push(edg.clone());
            m_cmd.lock().unwrap().entity(ent).insert(VirtualPathMember);
        }
    });

    std::mem::swap(&mut virt_path.edges, &mut scratch);
}
