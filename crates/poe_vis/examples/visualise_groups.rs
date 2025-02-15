use std::{thread, time::Duration};

use bevy::{math::Vec3, utils::hashbrown::HashMap};
use common::quick_tree_with_raw;
use poe_tree::{
    get_circle_radius,
    type_wrappings::{GroupId, NodeId},
};
use reqwest::blocking::Client;
mod common;

fn main() {
    let (tree, data) = quick_tree_with_raw();
    let client = Client::new();

    // Collect groups.
    let groups: HashMap<GroupId, Vec3> = data
        .get("passive_tree")
        .and_then(|t| t.get("groups"))
        .and_then(|g| g.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(gid, gval)| {
                    let gid = gid.parse::<NodeId>().ok()?;
                    let x = gval.get("x")?.as_f64()? as f32;
                    let y = gval.get("y")?.as_f64()? as f32 * -1.0;
                    Some((gid, Vec3 { x, y, z: 100.0 }))
                })
                .collect()
        })
        .unwrap_or_default();

    // Assign each group a unique colour.
    let colours = common::get_colours(&client).unwrap();
    let mut group_colours = HashMap::new();
    let mut idx = 0;
    for gid in groups.keys() {
        group_colours.insert(*gid, colours[idx % colours.len()].clone());
        idx += 1;
    }

    // Group nodes by their parent (group id).
    let mut group_nodes: HashMap<GroupId, Vec<_>> = HashMap::new();
    for (nid, node) in tree.nodes.iter() {
        group_nodes
            .entry(node.parent)
            .or_default()
            .push((nid, node));
    }

    let mut prev = Vec3::default();
    groups.iter().for_each(|(gid, grp_pos)| {
        if let Some(nodes) = group_nodes.get(gid) {
            // Group origin pos with fixed z
            let group_origin = Vec3 {
                x: grp_pos.x,
                y: grp_pos.y,
                z: 10.0,
            };
            // Move from prev to group origin
            if let Err(e) = smooth_move_camera2(&client, prev, group_origin) {
                eprintln!("{e}");
            }
            // Draw group circle
            let (_nid, rep) = nodes[0];
            let group_radius = get_circle_radius(rep.radius, rep.position, &rep.parent);
            let col = group_colours.get(gid).unwrap();
            common::draw_circle(&client, group_radius, *grp_pos, col, 500000);

            prev = group_origin;

            // Compute average position of nodes, placed.
            let (sum, count) = nodes
                .iter()
                .fold((Vec3::default(), 0), |(acc, cnt), (nid, _)| {
                    let pos = common::get_node_position(&client, **nid);
                    (
                        Vec3 {
                            x: acc.x + pos.x,
                            y: acc.y + pos.y,
                            z: acc.z + pos.z,
                        },
                        cnt + 1,
                    )
                });
            let avg = Vec3 {
                x: sum.x / count as f32,
                y: sum.y / count as f32,
                z: 10.0,
            };

            // Move from group origin to avg node position
            if let Err(e) = smooth_move_camera2(&client, group_origin, avg) {
                eprintln!("{e}");
            }
            prev = avg;

            // Draw each node's circle
            nodes.into_iter().for_each(|(nid, _)| {
                let pos = common::get_node_position(&client, **nid);
                common::draw_circle(&client, 92.0, pos, col, 500000);
                thread::sleep(Duration::from_millis(38));
            });

            // Move back from avg to group origin
            if let Err(e) = smooth_move_camera2(&client, avg, group_origin) {
                eprintln!("{e}");
            }
            prev = group_origin;

            thread::sleep(Duration::from_millis(300));
        }
    });
}

fn smooth_move_camera(
    client: &Client,
    start: Vec3,
    end: Vec3,
) -> Result<(), Box<dyn std::error::Error>> {
    let steps = 60; // 60 steps for 0.5s duration (~8.3ms per step)
    let sleep_time = std::time::Duration::from_millis(500) / steps;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let pos = Vec3 {
            x: start.x + (end.x - start.x) * t,
            y: start.y + (end.y - start.y) * t,
            z: start.z + (end.z - start.z) * t,
        };
        common::move_camera(client, pos)?;
        std::thread::sleep(sleep_time);
    }
    Ok(())
}
fn smooth_move_camera2(
    client: &Client,
    start: Vec3,
    end: Vec3,
) -> Result<(), Box<dyn std::error::Error>> {
    let steps = 60;
    let sleep_time = std::time::Duration::from_millis(500) / steps;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        // let eased = t * t * (3.0 - 2.0 * t); // ease in-out cubic
        let eased = 0.5 * (1.0 - (std::f32::consts::PI * t).cos());

        let pos = Vec3 {
            x: start.x + (end.x - start.x) * eased,
            y: start.y + (end.y - start.y) * eased,
            z: start.z + (end.z - start.z) * eased,
        };
        common::move_camera(client, pos)?;
        std::thread::sleep(sleep_time);
    }
    Ok(())
}
