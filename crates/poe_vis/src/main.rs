use bevy::prelude::*;

fn main() {
    let crate_name = env!("CARGO_PKG_NAME").replace('-', "_");
    // let log_filter = format!("{}=trace", crate_name);
    let log_filter = format!("trace");

    App::new()
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
