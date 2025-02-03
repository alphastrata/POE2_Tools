use bevy::prelude::*;

#[allow(unused_imports)]
use tracing_subscriber::{fmt::format::FmtSpan, Layer, Registry};
fn custom_log_formatting(_app: &mut App) -> Option<Box<dyn Layer<Registry> + Send + Sync>> {
    Some(
        tracing_subscriber::fmt::Layer::default()
            // .with_file(true)
            // .with_line_number(true)
            // .with_span_events(FmtSpan::ACTIVE)
            .boxed(),
    )
}

fn main() {
    let log_filter = {
        #[cfg(debug_assertions)]
        let crate_name = env!("CARGO_PKG_NAME").replace('-', "_");

        //NOTE: some may prefer this to the verbosely typing out what you want from the logs
        //in a terminal...
        #[cfg(debug_assertions)]
        let log_filter = format!("error,poe_tree::pathfinding=debug,{}=debug", crate_name);

        #[cfg(not(debug_assertions))]
        let log_filter = format!("error");
        log_filter
    };

    App::new()
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .add_plugins((
            DefaultPlugins.set(bevy::log::LogPlugin {
                // filter: log_filter,
                custom_layer: custom_log_formatting,
                ..default()
            }),
            MeshPickingPlugin,
        ))
        .add_plugins(poe_vis::PoeVis)
        .run();
}
