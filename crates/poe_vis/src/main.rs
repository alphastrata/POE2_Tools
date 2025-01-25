use bevy::prelude::*;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{Layer, Registry};

// Define a function matching the required signature
fn create_custom_layer(_app: &mut App) -> Option<Box<dyn Layer<Registry> + Send + Sync>> {
    Some(
        tracing_subscriber::fmt::Layer::default()
            .with_file(true)
            .with_line_number(true)
            .with_span_events(FmtSpan::ACTIVE)
            .boxed(),
    )
}

fn main() {
    let crate_name = env!("CARGO_PKG_NAME").replace('-', "_");
    // let log_filter = format!("{}=trace", crate_name);
    let log_filter = format!("error,{}=trace", crate_name);

    App::new()
        .add_plugins((
            DefaultPlugins.set(bevy::log::LogPlugin {
                filter: log_filter,
                custom_layer: create_custom_layer,
                ..default()
            }),
            MeshPickingPlugin,
        ))
        .add_plugins(poe_vis::PoeVis)
        .run();
}
