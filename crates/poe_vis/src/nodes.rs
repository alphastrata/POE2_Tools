use bevy::color::palettes::tailwind::{self, *};
use bevy::prelude::*;
use poe_tree::calculate_world_position;
use poe_tree::type_wrappings::NodeId;
use poe_tree::PassiveTree; // Add this import

#[derive(Debug, Clone, Component)]
pub struct PoeNode {
    pub color: Color,
    pub filled: bool,
    pub active: bool,
    pub node_id: NodeId,
}

// Add Resource derive for PassiveTree
#[derive(Resource, Debug, Clone)]
pub struct PassiveTreeWrapper {
    pub tree: PassiveTree,
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
        .add_systems(Startup, (spawn_nodes, spawn_edges, adjust_node_sizes));
    }
}
// Add this helper function for ColorMaterial

#[derive(Component)]
struct NodeMarker; // Marker component for nodes

#[derive(Resource)]
struct NodeScaling {
    min_scale: f32,
    max_scale: f32,
    base_radius: f32,
}

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
    let black_matl = materials.add(Color::BLACK);

    let node_radius = scaling.base_radius;
    let hollow_radius = scaling.base_radius * 0.75;

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
                NodeMarker,
            ))
            .observe(update_color_material_on::<Pointer<Over>>(
                hover_matl.clone(),
            ))
            .observe(update_color_material_on::<Pointer<Out>>(
                normal_matl.clone(),
            ))
            .observe(update_color_material_on::<Pointer<Down>>(
                pressed_matl.clone(),
            ))
            .observe(update_color_material_on::<Pointer<Up>>(hover_matl.clone()));

        // Non-interactive hollow center
        commands.spawn((
            Mesh2d(meshes.add(Circle::new(hollow_radius))),
            MeshMaterial2d(black_matl.clone()),
            Transform::from_translation(position + Vec3::Z * 0.1),
        ));
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

    for edge in &tree.tree.edges {
        // Grab both connected nodes:
        let (Some(start_node), Some(end_node)) = (
            tree.tree.nodes.get(&edge.start),
            tree.tree.nodes.get(&edge.end),
        ) else {
            // If either is missing, skip
            continue;
        };

        // Get each node’s parent group:
        let (Some(start_group), Some(end_group)) = (
            tree.tree.groups.get(&start_node.parent),
            tree.tree.groups.get(&end_node.parent),
        ) else {
            // If either group is missing, skip
            continue;
        };

        // Calculate world positions for both node centers:
        let start_pos =
            calculate_world_position(start_group, start_node.radius, start_node.position);
        let end_pos = calculate_world_position(end_group, end_node.radius, end_node.position);

        // Convert to Vec2 for geometry:
        let start = Vec2::new(start_pos.0, start_pos.1);
        let end = Vec2::new(end_pos.0, end_pos.1);

        // Vector from start to end:
        let delta = end - start;
        let width = delta.length();

        // Angle around Z = atan2(dy, dx)
        let angle = delta.y.atan2(delta.x);

        // Midpoint of the two nodes:
        let midpoint = start.lerp(end, 0.5);

        // Debug/logging
        log::warn!(
            "Edge: start_node: {:?}, end_node: {:?}, start: {:?}, end: {:?}, length: {:?}",
            start_node.node_id,
            end_node.node_id,
            start,
            end,
            width
        );

        // Spawn a rectangle that spans from start to end:
        //  - translation at the midpoint
        //  - rotation to align with delta
        //  - scale in X = 'length', Y = 'edge_thickness' if you’re using a custom mesh,
        //    or pass them to a Rectangle struct that’s created accordingly
        commands
            .spawn((
                // Provide the mesh (a rectangle) with correct length + thickness
                Mesh2d(meshes.add(Rectangle::new(width, height))),
                // Basic color material
                MeshMaterial2d(materials.add(edge_color)),
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
            .observe(update_color_material_on::<Pointer<Out>>(
                normal_matl.clone(),
            ))
            .observe(update_color_material_on::<Pointer<Down>>(
                pressed_matl.clone(),
            ))
            .observe(update_color_material_on::<Pointer<Up>>(hover_matl.clone()));

        // You can remove the second spawn if you only need one entity per edge
    }
}
