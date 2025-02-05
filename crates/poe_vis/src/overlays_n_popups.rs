use bevy::prelude::*;
use poe_tree::type_wrappings::NodeId;
use std::sync::{Arc, Mutex};

use crate::{
    components::*,
    events::{EdgeColourReq, NodeColourReq},
    materials::GameMaterials,
    resources::ActiveCharacter,
    PassiveTreeWrapper,
};

pub struct OverlaysAndPopupsPlugin;
impl Plugin for OverlaysAndPopupsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (virtual_path_to_node_under_cursor, show_node_info))
            .add_systems(Startup, spawn_hover_text);
    }
}

fn virtual_path_to_node_under_cursor(
    mut node_colouriser: EventWriter<NodeColourReq>,
    mut edge_colouriser: EventWriter<EdgeColourReq>,
    character: Res<ActiveCharacter>,
    hovered: Query<(Entity, &Hovered, &NodeMarker), With<NodeInactive>>,
    edges: Query<(Entity, &EdgeMarker), With<EdgeInactive>>,
    materials: Res<GameMaterials>,
    tree: Res<PassiveTreeWrapper>,
) {
    // if the hovered not in the character.active,
    // take paths from the hovered to every note in active, settle on the shortest,
    // mark them for virtualPath.
    // mark the corresponding edges too.
    let tree = &**tree;
    let mut must_colour_edges = vec![];
    let targets: Vec<NodeId> = character.activated_node_ids.iter().map(|v| *v).collect();
    hovered.iter().for_each(|(ent, _hovered, nm)| {
        tree.shortest_to_from_any_of(**nm, &targets)
            .into_iter()
            .for_each(|hit| {
                node_colouriser.send(NodeColourReq(ent, materials.blue.clone()));
                must_colour_edges.push(hit);
            });
    });

    let edg_tx = Arc::new(Mutex::new(&mut edge_colouriser));
    edges.par_iter().for_each(|(ent, em)| {
        let (s, e) = em.as_tuple();
        if must_colour_edges.contains(&s) && must_colour_edges.contains(&e) {
            edg_tx
                .lock()
                .unwrap()
                .send(EdgeColourReq(ent, materials.blue.clone()));
        }
        //
    });

    //QUESTION: ok we've highlighted them... how do we stop?!
}

//TODO: insert the virtual path length's extension to reach this node.
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
