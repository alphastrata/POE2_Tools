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
                    let y = gval.get("y")?.as_f64()? as f32;
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
    // For each group: draw the group circle, then each node's circle,
    // then compute the average (center) of all node positions and move the camera there.
    groups.iter().for_each(|(gid, grp_pos)| {
        if let Some(nodes) = group_nodes.get(gid) {
            let mut cam = *grp_pos;
            cam.z = 10.0;
            if let Err(e) = smooth_move_camera2(&client, prev, cam) {
                eprintln!("{e}");
            }
            prev = cam;

            let (_nid, rep) = nodes[0];
            let group_radius = get_circle_radius(rep.radius, rep.position, &rep.parent);
            let col = group_colours.get(gid).unwrap();
            common::draw_circle(&client, group_radius, *grp_pos, col, 5000);

            nodes.into_iter().for_each(|(nid, _pnode)| {
                let pos = common::get_node_position(&client, **nid);
                common::draw_circle(&client, 92.0, pos, col, 5000);
                thread::sleep(Duration::from_millis(38));
            });

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
