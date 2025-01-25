#![allow(unused_imports, unused_must_use, unused_mut, dead_code)]
use bevy::prelude::*;



// Main function
fn main() {
    let crate_name = env!("CARGO_PKG_NAME").replace('-', "_");
    let log_filter = format!("{}=trace", crate_name);

    App::new()
        .insert_resource(poe_vis::nodes::PassiveTreeWrapper { tree: passive_tree })
        .add_plugins((
            DefaultPlugins.set(bevy::log::LogPlugin {
                filter: log_filter,
                ..Default::default()
            }),
            MeshPickingPlugin,
        ))
        .add_plugins(poe_vis::PoeVis)
        .run();
}
