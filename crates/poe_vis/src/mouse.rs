use bevy::prelude::*;

use crate::{
    components::*, consts::DEFAULT_HOVER_FADE_TIME, materials::GameMaterials, resources::*,
};

pub struct MouseControlsPlugin;

impl Plugin for MouseControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_node_clicks, hover_started, hover_ended));
    }
}

pub fn handle_node_clicks(
    mut commands: Commands,
    mut drag_state: ResMut<crate::camera::DragState>,
    root: Res<crate::resources::RootNode>,
    mut click_events: EventReader<Pointer<Down>>,
    nodes_query: Query<
        (
            Entity,
            &NodeMarker,
            Option<&NodeInactive>,
            Option<&NodeActive>,
        ),
        Or<(With<NodeInactive>, With<NodeActive>)>,
    >,
) {
    for event in click_events.read() {
        if let Ok((entity, _marker, inactive, active)) = nodes_query.get(event.target) {
            drag_state.active = false; // so that we don't mess up ppl's camera when activating/deactivating nodes.
            match (inactive, active) {
                (Some(_), None) => {
                    // Activate if root exists
                    if root.0.is_some() {
                        commands
                            .entity(entity)
                            .remove::<NodeInactive>()
                            .insert(NodeActive);
                    }
                }
                (None, Some(_)) => {
                    // Deactivate regardless of root
                    commands
                        .entity(entity)
                        .remove::<NodeActive>()
                        .insert(NodeInactive);
                }
                _ => unreachable!(),
            }
        }
    }
}

pub fn hover_started(
    mut commands: Commands,
    mut over_events: EventReader<Pointer<Over>>,
    query_nodes: Query<(
        Entity,
        &NodeMarker,
        &Transform,
        Option<&Hovered>,
        Option<&NodeActive>, // so you can decide color
    )>,
    game_materials: Res<GameMaterials>,
) {
    for ev in over_events.read() {
        if let Ok((entity, marker, transform, maybe_hovered, maybe_active)) =
            query_nodes.get(ev.target)
        {
            if maybe_hovered.is_none() {
                commands.entity(entity).insert(Hovered {
                    timer: Timer::from_seconds(DEFAULT_HOVER_FADE_TIME, TimerMode::Once),
                    base_scale: transform.scale.x,
                });
                // Pick color based on active or not
                if maybe_active.is_some() {
                    commands
                        .entity(entity)
                        .insert(MeshMaterial2d(game_materials.cyan.clone()));
                } else {
                    commands
                        .entity(entity)
                        .insert(MeshMaterial2d(game_materials.orange.clone()));
                }
            }
        }
    }
}
pub fn hover_ended(
    mut commands: Commands,
    mut out_events: EventReader<Pointer<Out>>,
    query_nodes: Query<(Entity, Option<&NodeActive>, Option<&Hovered>)>,
    game_materials: Res<GameMaterials>,
) {
    for ev in out_events.read() {
        if let Ok((entity, maybe_active, maybe_hovered)) = query_nodes.get(ev.target) {
            if maybe_hovered.is_some() {
                commands.entity(entity).remove::<Hovered>();
                // revert color
                if maybe_active.is_some() {
                    commands
                        .entity(entity)
                        .insert(MeshMaterial2d(game_materials.node_activated.clone()));
                } else {
                    commands
                        .entity(entity)
                        .insert(MeshMaterial2d(game_materials.node_base.clone()));
                }
            }
        }
    }
}
