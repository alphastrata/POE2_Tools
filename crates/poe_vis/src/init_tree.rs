use bevy::prelude::*;
use poe_tree::calculate_world_position_with_negative_y;

use crate::materials::GameMaterials;
use crate::{components::*, resources::*, PassiveTreeWrapper};

 pub(crate) struct TreeCanvasPlugin;

impl Plugin for TreeCanvasPlugin {
    fn build(&self, app: &mut App) {
        fn quick_tree() -> poe_tree::PassiveTree {
            let file = std::fs::File::open("data/POE2_Tree.json").unwrap();
            let reader = std::io::BufReader::new(file);
            let tree_data: serde_json::Value = serde_json::from_reader(reader).unwrap();
            let mut tree = poe_tree::PassiveTree::from_value(&tree_data).unwrap();

            tree.remove_hidden();
            tree
        }

        let tree = quick_tree();
        app.insert_resource(PassiveTreeWrapper { tree });
        app.add_systems(Startup, (spawn_nodes, spawn_edges));
    }
}

 fn spawn_nodes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<GameMaterials>,
    tree: Res<PassiveTreeWrapper>,
    scaling: Res<NodeScaling>,
) {
    let node_radius = scaling.base_radius;

    for (_, node) in tree.tree.nodes.iter() {
        let group = tree.tree.groups.get(&node.parent).unwrap();
        let (x, y) = calculate_world_position_with_negative_y(group, node.radius, node.position);

        commands.spawn((
            Mesh2d(meshes.add(Circle::new(node_radius))),
            MeshMaterial2d(materials.node_base.clone()),
            Transform::from_translation(Vec3::new(x, y, 0.0)),
            NodeMarker(node.node_id),
            NodeInactive,
        ));
    }
}

 fn spawn_edges(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<GameMaterials>,
    tree: Res<PassiveTreeWrapper>,
) {
    tree.tree.edges.iter().for_each(|edge| {
        let (start_node, end_node) = (
            tree.tree.nodes.get(&edge.start).unwrap(),
            tree.tree.nodes.get(&edge.end).unwrap(),
        );

        let (start_group, end_group) = (
            tree.tree.groups.get(&start_node.parent).unwrap(),
            tree.tree.groups.get(&end_node.parent).unwrap(),
        );

        let start_pos = calculate_world_position_with_negative_y(
            start_group,
            start_node.radius,
            start_node.position,
        );
        let end_pos =
            calculate_world_position_with_negative_y(end_group, end_node.radius, end_node.position);
        let start = Vec2::new(start_pos.0, start_pos.1);
        let end = Vec2::new(end_pos.0, end_pos.1);

        let delta = end - start;
        let width = delta.length();
        let height = 20.0;
        let angle = delta.y.atan2(delta.x);
        let midpoint = start.lerp(end, 0.5);

        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(width, height))),
            MeshMaterial2d(materials.edge_base.clone()),
            EdgeMarker((edge.start, edge.end)),
            Transform::from_translation(midpoint.extend(-0.01))
                .with_rotation(Quat::from_rotation_z(angle)),
            EdgeInactive,
        ));
    });
}
