use bevy::{math::Vec3, utils::hashbrown::HashMap};
use common::get_node_position;
use poe_tree::{
    get_circle_radius,
    type_wrappings::{GroupId, NodeId},
};
use reqwest::blocking::Client;
mod common;

fn main() {
    let (tree, data) = common::quick_tree_with_raw();
    let client = Client::new();

    // Group positions.
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

    // Get colours and assign each group a unique one.
    let colours = common::get_colours(&client).unwrap();
    let mut group_colours: HashMap<GroupId, String> = HashMap::new();
    let mut idx = 0;
    for gid in groups.keys() {
        group_colours.insert(*gid, colours[idx % colours.len()].clone());
        idx += 1;
    }

    groups.iter().for_each(|(gid, &grp_pos)| {
        if let Some((_, pnode)) = tree.nodes.iter().find(|(_, n)| n.parent == *gid) {
            let r = get_circle_radius(pnode.radius, pnode.position, &pnode.parent);
            let col = group_colours.get(gid).unwrap();
            common::draw_circle(&client, r, grp_pos, col, 60000);
        }
    });

    tree.nodes.iter().for_each(|(nid, pnode)| {
        if let Some(col) = group_colours.get(&pnode.parent) {
            let pos = get_node_position(&client, *nid);
            common::draw_circle(&client, 140.0, pos, col, 60000);
        }
    });
}
