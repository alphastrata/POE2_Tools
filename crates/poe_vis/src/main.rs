use bevy::prelude::*;
use poe_tree::calculate_world_position;
use poe_tree::PassiveTree; // Add this import

#[derive(Debug, Clone)]
pub struct PoeNode {
    pub position: Vec2,
    pub color: Color,
    pub filled: bool,
    pub active: bool,
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
        app.add_plugins(camera::PoeVisCameraPlugin)
            .add_systems(Startup, spawn_nodes);
    }
}
pub mod camera;

fn spawn_nodes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    tree: Res<PassiveTreeWrapper>,
) {
    let pale_blue = Color::srgb(0.6, 0.8, 1.0);
    let node_radius = 20.0;
    let hollow_radius = 15.0;

    for (_, node) in tree.tree.nodes.iter() {
        let group = tree.tree.groups.get(&node.parent).unwrap();
        let (x, y) = calculate_world_position(group, node.radius, node.position);
        println!("spawning node at {}, {}", x, y);
        let position = Vec3::new(x, y, 0.0);

        // Main node
        commands.spawn((
            Mesh2d(meshes.add(Circle::new(node_radius))),
            MeshMaterial2d(materials.add(pale_blue)),
            Transform::from_translation(position),
        ));

        // Hollow center
        commands.spawn((
            Mesh2d(meshes.add(Circle::new(hollow_radius))),
            MeshMaterial2d(materials.add(Color::BLACK)),
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
    let edge_color = Color::srgb(0.3, 0.3, 0.3); // Dark gray for edges
    let edge_thickness = 2.0;

    for edge in &tree.tree.edges {
        // Get both nodes from the edge
        let (Some(start_node), Some(end_node)) = (
            tree.tree.nodes.get(&edge.start),
            tree.tree.nodes.get(&edge.end),
        ) else {
            continue;
        };

        // Get parent groups for both nodes
        let (Some(start_group), Some(end_group)) = (
            tree.tree.groups.get(&start_node.parent),
            tree.tree.groups.get(&end_node.parent),
        ) else {
            continue;
        };

        // Calculate world positions for both ends of the edge
        let start_pos =
            calculate_world_position(start_group, start_node.radius, start_node.position);
        let end_pos = calculate_world_position(end_group, end_node.radius, end_node.position);

        // Create line segment between the points
        let start = Vec2::new(start_pos.0, start_pos.1);
        let end = Vec2::new(end_pos.0, end_pos.1);
        let delta = end - start;
        let length = delta.length();

        // Create rectangle primitive for the edge
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(length, edge_thickness))),
            MeshMaterial2d(materials.add(edge_color)),
            Transform {
                translation: start.lerp(end, 0.5).extend(0.0),
                rotation: Quat::from_rotation_z(delta.angle_to(Vec2::X)),
                ..default()
            },
        ));
    }
}
fn quick_tree() -> PassiveTree {
    let file = std::fs::File::open("data/POE2_Tree.json").unwrap();
    let reader = std::io::BufReader::new(file);
    let tree_data: serde_json::Value = serde_json::from_reader(reader).unwrap();
    let mut tree = PassiveTree::from_value(&tree_data).unwrap();

    tree.remove_hidden();
    tree
}
// Main function
fn main() {
    let passive_tree = quick_tree();

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PoeVis)
        .insert_resource(PassiveTreeWrapper { tree: passive_tree })
        .run();
}
