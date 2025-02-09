use bevy::prelude::*;
use poe_tree::calculate_world_position_with_negative_y;

use crate::consts::{EDGE_PLACEMENT_Z_IDX, NODE_PLACEMENT_Z_IDX};
use crate::materials::GameMaterials;
use crate::{components::*, resources::*, PassiveTreeWrapper};

// Remove once we've found the 'longest' edge we'll ever spawn.

use std::sync::{Arc, Mutex, OnceLock};

static SMALLEST_EDGE: OnceLock<Arc<Mutex<f32>>> = OnceLock::new();
static LARGEST_EDGE: OnceLock<Arc<Mutex<f32>>> = OnceLock::new();

fn init_globals() {
    SMALLEST_EDGE.get_or_init(|| Arc::new(Mutex::new(f32::MAX)));
    LARGEST_EDGE.get_or_init(|| Arc::new(Mutex::new(f32::MIN)));
}

pub(crate) struct TreeCanvasPlugin;

impl Plugin for TreeCanvasPlugin {
    fn build(&self, app: &mut App) {
        fn quick_tree() -> poe_tree::PassiveTree {
            let file = std::fs::File::open("data/POE2_Tree.json").unwrap();
            let reader = std::io::BufReader::new(file);
            let tree_data: serde_json::Value = serde_json::from_reader(reader).unwrap();
            #[allow(unusued_mut)]
            let mut tree = poe_tree::PassiveTree::from_value(&tree_data).unwrap();

            // tree.remove_hidden();
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
        app.add_systems(Startup, (spawn_nodes, spawn_edges));

        log::debug!("TreeCanvas plugin enabled");
    }
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
    init_globals();

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

        // TODO: remove when we know the const.
        let edge_dist = start.distance(end);
        if let Some(smallest) = SMALLEST_EDGE.get() {
            let mut smallest_lock = smallest.lock().unwrap();
            if edge_dist < *smallest_lock {
                *smallest_lock = edge_dist;
                dbg!("Updated smallest:", *smallest_lock);
            }
        }

        if let Some(largest) = LARGEST_EDGE.get() {
            let mut largest_lock = largest.lock().unwrap();
            if edge_dist > *largest_lock {
                *largest_lock = edge_dist;
                dbg!("Updated largest:", *largest_lock);
            }
        }

        if edge_dist > 5_000.0 {
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
