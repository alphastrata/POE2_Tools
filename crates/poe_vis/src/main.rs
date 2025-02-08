use bevy::prelude::*;

use tracing_subscriber::{Layer, Registry};
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
    let log_filter = { "poe_vis::ui=trace,poe_vis::background_services=debug".to_string() };

    App::new()
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .add_plugins((
            DefaultPlugins.set(bevy::log::LogPlugin {
                filter: log_filter,
                custom_layer: custom_log_formatting,
                ..default()
            }),
            MeshPickingPlugin,
        ))
        .add_plugins(poe_vis::PoeVis)
        .run();
}
