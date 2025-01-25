use super::{GameMaterials, NodeScaling};
use crate::components::{Hovered, NodeActive, NodeInactive, NodeMarker};
use crate::events::ScaleNode;
use crate::nodes::PassiveTreeWrapper;
use bevy::prelude::*;


// Instead of directly changing Transform/Material, fire events:
pub fn handle_hovered_scaling(
    scaling: Res<NodeScaling>,
    query: Query<(Entity, &Hovered)>,
    mut scale_writer: EventWriter<ScaleNode>,
){
    for (entity, _hovered) in query.iter() {
        let new_scale = scaling.base_radius * scaling.hover_multiplier;
        scale_writer.send(ScaleNode(entity, new_scale));
    }
}

pub fn handle_highlighted_active_nodes(
    scaling: Res<NodeScaling>,
    query: Query<Entity, With<NodeActive>>,
    mut scale_writer: EventWriter<ScaleNode>,
){
    for entity in query.iter() {
        let new_scale = scaling.base_radius * scaling.hover_multiplier;
        scale_writer.send(ScaleNode(entity, new_scale));
    }
}

pub fn revert_hovered_nodes(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Hovered, &mut Transform), With<Hovered>>,
    game_materials: Res<GameMaterials>,
    mut scale_writer: EventWriter<ScaleNode>,
    mut colour_writer: EventWriter<ColourNode>,
){
    for (entity, mut hovered, mut transform) in query.iter_mut() {
        hovered.timer.tick(time.delta());
        if hovered.timer.finished() {
            // revert scale
            scale_writer.send(ScaleNode(entity, hovered.base_scale));
            // revert color
            colour_writer.send(ColourNode(entity, game_materials.node_base.clone()));
            // also reflect immediate scale so system doesn't flicker
            transform.scale = Vec3::splat(hovered.base_scale);
            commands.entity(entity).remove::<Hovered>();
        }
    }
}


