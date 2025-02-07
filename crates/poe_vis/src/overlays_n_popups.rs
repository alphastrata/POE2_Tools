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

        app.add_systems(Update, show_node_info).add_systems(
            Startup,
            (
                spawn_hover_text,
                //
            ),
        );
        app.add_systems(
            Update,
            (
                debug_num_nodes_in_virt_path,
                scan_for_hovered.run_if(resource_exists::<ActiveCharacter>),
            ),
        );

        log::debug!("OverlaysAndPopups plugin enabled.");
    }
}

fn debug_num_nodes_in_virt_path(query: Query<(Entity, &NodeMarker), With<VirtualPathMember>>) {
    log::debug!("Members in VP: {}", query.iter().count());
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

fn scan_for_hovered(mut commands: Commands, hovered: Query<(Entity, &NodeMarker), With<Hovered>>) {
    hovered.iter().for_each(|(ent, _nm)| {
        commands.entity(ent).insert(VirtualPathMember);
    });
}
