use bevy::prelude::*;
use poe_tree::calculate_world_position;
use poe_tree::PassiveTree;

use crate::components::Active;
use crate::components::EdgeMarker;
use crate::components::NodeMarker;
use crate::config::parse_hex_color;
use crate::config::UserConfig;

// Add Resource derive for PassiveTree
#[derive(Resource, Debug, Clone)]
pub struct PassiveTreeWrapper {
    pub tree: PassiveTree,
}

#[derive(Resource)]
struct NodeScaling {
    min_scale: f32,
    max_scale: f32,
    base_radius: f32,
}
// Plugin to display nodes
pub struct PoeVis;

impl Plugin for PoeVis {
    fn build(&self, app: &mut App) {
        app.insert_resource(NodeScaling {
            min_scale: 1.0,    // Nodes can shrink to 50% size
            max_scale: 4.0,    // Nodes can grow to 200% size
            base_radius: 40.0, // Should match your node radius
        })
        .add_plugins(crate::camera::PoeVisCameraPlugin)
        .add_systems(Startup, (spawn_nodes, spawn_edges, adjust_node_sizes))
        .add_systems(
            Update,
            (
                handle_node_clicks,
                update_active_edges,
                update_active_materials,
            ),
        );
    }
}

// System to handle node clicks
fn handle_node_clicks(
    mut click_events: EventReader<Pointer<Click>>,
    mut node_query: Query<&mut Active, With<NodeMarker>>,
) {
    for event in click_events.read() {
        if let Ok(mut active) = node_query.get_mut(event.entity()) {
            active.0 = !active.0;
        }
    }
}

// System to update connected edges
fn update_active_edges(
    node_query: Query<(&NodeMarker, &Active), Changed<Active>>,
    mut edge_query: Query<(&EdgeMarker, &mut Active)>,
    tree: Res<PassiveTreeWrapper>,
) {
    for (node_marker, node_active) in &node_query {
        for (edge_marker, mut edge_active) in &mut edge_query {
            // Check if edge is connected to this node
            if edge_marker.0 == node_marker.0 || edge_marker.1 == node_marker.0 {
                edge_active.0 = node_active.0;
            }
        }
    }
}

// System to update materials based on active state
fn update_active_materials(
    config: Res<UserConfig>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    activated_materials: Res<ActivatedMaterials>,
    mut node_query: Query<(&Active, &mut MeshMaterial2d), Changed<Active>>,
    mut edge_query: Query<(&Active, &mut MeshMaterial2d), Changed<Active>>,
) {
    // Update nodes
    for (active, mut material) in &mut node_query {
        material.0 = if active.0 {
            activated_materials.node.clone()
        } else {
            // Get original material from config
            materials.add(parse_hex_color(&config.colors["all_nodes"]))
        };
    }

    // Update edges
    for (active, mut material) in &mut edge_query {
        material.0 = if active.0 {
            activated_materials.edge.clone()
        } else {
            // Get original edge material from config
            materials.add(parse_hex_color(&config.colors["all_nodes"]))
        };
    }
}
// Add this helper function for ColorMaterial

/// Adjust each node’s `Transform.scale` so it doesn’t get too big or too small on screen.
fn adjust_node_sizes(
    camera_query: Query<&OrthographicProjection, With<Camera2d>>,
    mut node_query: Query<&mut Transform, With<NodeMarker>>,
) {
    // If you only have one main camera, just get_single()
    if let Ok(projection) = camera_query.get_single() {
        // By default, a larger `projection.scale` means you are "zoomed out"
        // so items appear smaller on screen, and vice versa.

        // For example, if you want the node scale to be simply the inverse:
        //   "1.0 / projection.scale"
        // you can then clamp that to keep it from vanishingly small or huge:
        let unscaled = 1.0 / projection.scale;
        let final_scale = unscaled.clamp(0.02, 2.0);
        // tweak these clamp values to taste
        log::debug!("{}", final_scale);

        for mut transform in &mut node_query {
            transform.scale = Vec3::splat(final_scale);
        }
    }
}

/// The closure returned by this function can be used to observe pointer events
/// (Pointer<Over>, Pointer<Out>, etc.) and update the `ColorMaterial` accordingly,
/// while also printing debug info.
fn update_color_material_on<E>(
    new_material: Handle<ColorMaterial>,
) -> impl Fn(
    // The event/trigger type
    Trigger<E>,
    // Query for (MeshMaterial2d<ColorMaterial>, Transform)
    Query<(&mut MeshMaterial2d<ColorMaterial>, &Transform)>,
    // Access to the global Assets to look up the color from our `new_material` handle
    Res<Assets<ColorMaterial>>,
) + 'static {
    move |trigger, mut query, materials| {
        if let Ok((mut material, transform)) = query.get_mut(trigger.entity()) {
            // If we can find the underlying ColorMaterial in the asset store,
            // print out the actual color being applied
            if let Some(mat) = materials.get(&new_material) {
                log::debug!(
                    "Entity: {:?} -> applying color: {:?}, at world position: {:?}",
                    trigger.entity(),
                    mat.color,
                    transform.translation
                );
            } else {
                // If we can't find it for some reason, at least show the handle ID
                log::debug!(
                    "Entity: {:?} -> applying unknown color handle: {:?} at position: {:?}",
                    trigger.entity(),
                    new_material.id(),
                    transform.translation
                );
            }

            // Finally apply the material
            material.0 = new_material.clone();
        }
    }
}

// Modified node spawning with proper ColorMaterial handling
fn spawn_nodes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    tree: Res<PassiveTreeWrapper>,
    scaling: Res<NodeScaling>,
) {
    // Create all material handles upfront
    let normal_matl = materials.add(Color::srgb(0.6, 0.8, 1.0)); // Pale blue
    let hover_matl = materials.add(Color::srgb(0.0, 0.5, 0.5)); // Green
    let pressed_matl = materials.add(Color::srgb(1.0, 1.0, 0.0)); // Yellow

    // let black_matl = materials.add(Color::BLACK);

    let node_radius = scaling.base_radius;
    // let hollow_radius = scaling.base_radius * 0.75;

    for (_, node) in tree.tree.nodes.iter() {
        let group = tree.tree.groups.get(&node.parent).unwrap();
        let (x, y) = calculate_world_position(group, node.radius, node.position);
        let position = Vec3::new(x, y, 0.0);

        // Interactive main node
        commands
            .spawn((
                Mesh2d(meshes.add(Circle::new(node_radius))),
                MeshMaterial2d(normal_matl.clone()),
                Transform::from_translation(position),
                NodeMarker(node.node_id),
                Active(false),
            ))
            .observe(update_color_material_on::<Pointer<Over>>(
                hover_matl.clone(),
            ))
            // .observe(update_color_material_on::<Pointer<Out>>(
            //     normal_matl.clone(),
            // ))
            .observe(update_color_material_on::<Pointer<Down>>(
                pressed_matl.clone(),
            ));
        // .observe(update_color_material_on::<Pointer<Up>>(hover_matl.clone()));

        // // Non-interactive hollow center
        // commands.spawn((
        //     NodeMarker,
        //     Mesh2d(meshes.add(Circle::new(hollow_radius))),
        //     MeshMaterial2d(black_matl.clone()),
        //     Transform::from_translation(position + Vec3::Z * 0.1),
        // ));
    }
}

fn spawn_edges(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    tree: Res<PassiveTreeWrapper>,
) {
    log::info!("spawn_edges function was called");

    let edge_color = Color::srgb(0.3, 0.3, 0.3); // Dark gray for edges
    let height = 2.0;

    // Optional highlight materials:
    let normal_matl = materials.add(Color::srgb(0.6, 0.8, 1.0)); // Pale blue
    let hover_matl = materials.add(Color::srgb(0.0, 0.5, 0.5)); // Green
    let pressed_matl = materials.add(Color::srgb(1.0, 1.0, 0.0)); // Yellow

    tree.tree.edges.iter().for_each(|edge| {
        let (start_node, end_node) = (
            tree.tree.nodes.get(&edge.start).unwrap(),
            tree.tree.nodes.get(&edge.end).unwrap(),
        );

        let (start_group, end_group) = (
            tree.tree.groups.get(&start_node.parent).unwrap(),
            tree.tree.groups.get(&end_node.parent).unwrap(),
        );

        let start_pos =
            calculate_world_position(start_group, start_node.radius, start_node.position);
        let end_pos = calculate_world_position(end_group, end_node.radius, end_node.position);
        let start = Vec2::new(start_pos.0, start_pos.1);
        let end = Vec2::new(end_pos.0, end_pos.1);

        // Vector from start to end:
        let delta = end - start;
        let width = delta.length();

        // Angle around Z = atan2(dy, dx)
        let angle = delta.y.atan2(delta.x);

        // Midpoint of the two nodes:
        let midpoint = start.lerp(end, 0.5);

        commands
            .spawn((
                // Provide the mesh (a rectangle) with correct length + thickness
                Mesh2d(meshes.add(Rectangle::new(width, height))),
                // Basic color material
                MeshMaterial2d(materials.add(edge_color)),
                EdgeMarker((edge.start, edge.end)),
                Active(false),
                // Transform: position and rotation
                Transform {
                    translation: midpoint.extend(0.0),
                    rotation: Quat::from_rotation_z(angle),
                    ..default()
                },
            ))
            // Optional: tie into pointer-based color changes
            .observe(update_color_material_on::<Pointer<Over>>(
                hover_matl.clone(),
            ))
            // .observe(update_color_material_on::<Pointer<Out>>(
            //     normal_matl.clone(),
            // ))
            .observe(update_color_material_on::<Pointer<Down>>(
                pressed_matl.clone(),
            ));
        // .observe(update_color_material_on::<Pointer<Up>>(hover_matl.clone()));
    });
}
