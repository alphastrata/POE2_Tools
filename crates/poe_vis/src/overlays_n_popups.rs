use bevy::prelude::*;
use poe_tree::{character, type_wrappings::NodeId};
use std::sync::{Arc, Mutex};

use crate::{
    background_services::parse_tailwind_color,
    components::*,
    events::{DrawCircleReq, DrawRectangleReq},
    materials::GameMaterials,
    resources::{ActiveCharacter, VirtualPath},
    PassiveTreeWrapper,
};

pub struct OverlaysAndPopupsPlugin;
impl Plugin for OverlaysAndPopupsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(VirtualPath::default());

        app.add_systems(
            Startup,
            (
                spawn_hover_text,
                //
            ),
        );
        app.add_systems(
            Update,
            (
                show_node_info,
                debug_num_nodes_in_virt_path,
                draw_circles,
                draw_rectangles,
                cleanup_expired_timers,
                debug_timers,
                scan_for_hovered.run_if(resource_exists::<ActiveCharacter>),
            ),
        );

        log::debug!("OverlaysAndPopups plugin enabled.");
    }
}

fn debug_timers(mut query: Query<(Entity, &mut UIGlyph)>, time: Res<Time>) {
    query.iter_mut().for_each(|(_ent, mut glyph)| {
        glyph.tick(time.delta());
    });
}

fn cleanup_expired_timers(mut commands: Commands, query: Query<(Entity, &UIGlyph), With<UIGlyph>>) {
    query.into_iter().for_each(|(ent, glyph)| {
        if glyph.finished() {
            commands.entity(ent).despawn_recursive();
        }
    });
}
fn draw_rectangles(
    mut commands: Commands,
    mut draw: EventReader<DrawRectangleReq>,
    mut meshes: ResMut<Assets<Mesh>>,
    game_materials: Res<GameMaterials>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    draw.read().for_each(|r| {
        let DrawRectangleReq {
            half_size,
            origin,
            glyph,
            mat,
        } = r;
        let mat = match game_materials.other.get(mat) {
            Some(mat) => mat.clone_weak(),
            None => {
                let mat_from_str = parse_tailwind_color(mat);
                materials.add(mat_from_str)
            }
        };
        let g = glyph.clone();

        commands.spawn(g).with_child((
            Mesh2d(meshes.add(Rectangle::new(half_size.x * 2.0, half_size.y * 2.0))),
            MeshMaterial2d(mat),
            Transform::from_translation(*origin),
        ));
    });
}

fn draw_circles(
    mut commands: Commands,
    mut draw: EventReader<DrawCircleReq>,
    mut meshes: ResMut<Assets<Mesh>>,
    game_materials: Res<GameMaterials>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    draw.read().for_each(|r| {
        let DrawCircleReq {
            glyph,
            mat,
            radius,
            origin,
        } = r;

        let mat = match game_materials.other.get(mat) {
            Some(mat) => mat.clone_weak(),
            None => {
                let mat_from_str = parse_tailwind_color(mat);
                materials.add(mat_from_str)
            }
        };
        let g = glyph.clone();
        commands
            .spawn((g, Transform::from_translation(*origin)))
            .with_child((
                Mesh2d(meshes.add(Annulus::new(*radius * 0.95, *radius))),
                MeshMaterial2d(mat),
            ));
    });
}

fn debug_num_nodes_in_virt_path(query: Query<(Entity, &NodeMarker), With<VirtualPathMember>>) {
    log::debug!("Members in VP: {}", query.iter().count());
}

fn show_node_info(
    windows: Query<&Window>,
    hovered: Query<(&Hovered, &NodeMarker, Option<&NodeActive>)>,
    mut hover_text_query: Query<(&mut Node, &mut Text), With<NodeHoverText>>,
    virt_path: Query<&NodeMarker, With<VirtualPathMember>>,
    tree: Res<PassiveTreeWrapper>,
    active_character: Res<ActiveCharacter>,
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
            let vp_count = virt_path.iter().count();
            found_info = Some(format!(
                "\nNode {}:{}\nTotalCost :{}\nDelta: {}",
                node_info.node_id,
                node_info.name,
                vp_count,
                (vp_count as isize - active_character.activated_node_ids.len() as isize)
            ));
            break;
        }
    }

    // Update the text content if we found information
    if let Some(info) = found_info {
        text.0 = info;
    }

    // Update the node's position in screen space
    if let Some(cursor_pos) = windows.single().cursor_position() {
        //TODO: get a % of the screen to offset this by, as exactly on the pointer is kinda garbage..
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
