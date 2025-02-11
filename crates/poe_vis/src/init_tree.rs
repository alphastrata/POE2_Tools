use bevy::prelude::*;
use poe_tree::calculate_world_position_with_negative_y;
use poe_tree::consts::CHAR_START_NODES;

use crate::consts::{EDGE_PLACEMENT_Z_IDX, NODE_PLACEMENT_Z_IDX};
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
            // If you DON'T WANT THE ASCENDENCIES.
            tree.remove_hidden();

            // CHAR_START_NODES.iter().for_each(|nid| {
            //     _ = tree.nodes.remove(nid);
            // });

            tree
        }

        app.insert_resource(NodeScaling {
            min_scale: 1.0,         // Nodes can shrink
            max_scale: 6.0,         // Nodes can grow
            base_radius: 7.2,       //
            hover_multiplier: 1.06, // Nodes that are hovered are increased by %3 of their size
            hover_fade_time: 0.120,
        });

        let tree = quick_tree();
        log::debug!("Tree parsing complete.");

        app.insert_resource(PassiveTreeWrapper { tree });

        log::debug!("Tree in ECS");
        app.add_systems(
            Startup,
            (
                // spawn_bg_circles, //TODO:
                spawn_nodes,
                spawn_edges,
            ),
        );

        log::debug!("TreeCanvas plugin enabled");
    }
}

fn spawn_bg_circles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<GameMaterials>,
) {
    let radius = 11_000.0;
    let circle_mesh = Circle::new(radius);

    commands.spawn((
        Mesh2d(meshes.add(circle_mesh)), // Large circle mesh
        MeshMaterial2d(materials.background.clone_weak()), // Use appropriate material
        Transform::from_translation(Vec3::new(0.0, 0.0, -1_000.0)), // Position behind the tree
    ));
}

fn spawn_nodes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<GameMaterials>,
    tree: Res<PassiveTreeWrapper>,
    scaling: Res<NodeScaling>,
) {
    log::debug!("Spawning nodes...");

    let node_radius = scaling.base_radius;

    tree.tree.nodes.iter().for_each(|(_, node)| {
        let group = tree.tree.groups.get(&node.parent).unwrap();
        let (x, y) = calculate_world_position_with_negative_y(group, node.radius, node.position);

        commands.spawn((
            Mesh2d(meshes.add(Circle::new(node_radius))),
            MeshMaterial2d(materials.node_base.clone_weak()),
            Transform::from_translation(Vec3::new(x, y, NODE_PLACEMENT_Z_IDX)),
            NodeMarker(node.node_id),
            NodeInactive,
            Skill(node.as_passive_skill(&tree).clone()),
        ));
    });

    log::debug!("All nodes spawned.");
}

fn spawn_edges(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<GameMaterials>,
    tree: Res<PassiveTreeWrapper>,
) {
    tree.tree.edges.iter().for_each(|edge| {
        let (start_node, end_node) = match (
            tree.tree.nodes.get(&edge.start),
            tree.tree.nodes.get(&edge.end),
        ) {
            (Some(start), Some(end)) => (start, end),
            (start, end) => {
                log::error!("Failed to fetch nodes, start: {:?}, end: {:?}", start, end);
                return;
            }
        };

        let (start_group, end_group) = match (
            tree.tree.groups.get(&start_node.parent),
            tree.tree.groups.get(&end_node.parent),
        ) {
            (Some(start_group), Some(end_group)) => (start_group, end_group),
            (start_group, end_group) => {
                log::error!(
                    "Failed to fetch groups, start_group: {:?}, end_group: {:?}",
                    start_group,
                    end_group
                );
                return;
            }
        };

        // TODO: work out how to connect the nodes with ArcSegment(s) instead
        // of straight lines, ideally concave/convex as is appropriate from whatever the fuck
        // algo they(GGG) are using
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

        // Don't render edges if they're too big.
        if start.distance(end) > 5_000.0 {
            log::warn!("Edge is so big, it's for an ascendency don't render it.");
            return;
        }

        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(width, height))),
            MeshMaterial2d(materials.edge_base.clone_weak()),
            EdgeMarker(edge.start, edge.end),
            Transform::from_translation(midpoint.extend(EDGE_PLACEMENT_Z_IDX))
                .with_rotation(Quat::from_rotation_z(angle)),
            EdgeInactive,
        ));
    });
}
