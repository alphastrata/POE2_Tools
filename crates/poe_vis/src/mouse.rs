use bevy::prelude::*;

use crate::{
    components::*,
    consts::DEFAULT_HOVER_FADE_TIME,
    events::{NodeActivationReq, NodeColourReq, NodeDeactivationReq},
    materials::GameMaterials,
    resources::*,
};

pub struct MouseControlsPlugin;

impl Plugin for MouseControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_node_clicks, hover_started, hover_ended));
    }
}

pub fn handle_node_clicks(
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
    mut path_repair: ResMut<PathRepairRequired>,
    mut activate_events: EventWriter<NodeActivationReq>,
    mut deactivate_events: EventWriter<NodeDeactivationReq>,
) {
    for event in click_events.read() {
        if let Ok((_entity, marker, inactive, active)) = nodes_query.get(event.target) {
            drag_state.active = false;

            match (inactive, active) {
                (Some(_), None) => {
                    if root.0.is_some() {
                        activate_events.send(NodeActivationReq(marker.0));
                    }
                    path_repair.request_path_repair();
                }
                (None, Some(_)) => {
                    deactivate_events.send(NodeDeactivationReq(marker.0));
                    path_repair.request_path_repair();
                }
                _ => unreachable!(),
            }
        }
    }
}

pub fn hover_started(
    mut commands: Commands,
    mut over_events: EventReader<Pointer<Over>>,
    mut colour_events: EventWriter<NodeColourReq>,
    query_nodes: Query<(
        Entity,
        &NodeMarker,
        &Transform,
        Option<&Hovered>,
        Option<&NodeActive>,
    )>,
    game_materials: Res<GameMaterials>,
) {
    over_events.read().for_each(|ev| {
        if let Ok((entity, _marker, transform, maybe_hovered, maybe_active)) =
            query_nodes.get(ev.target)
        {
            if maybe_hovered.is_none() {
                commands.entity(entity).insert(Hovered {
                    timer: Timer::from_seconds(DEFAULT_HOVER_FADE_TIME, TimerMode::Once),
                    base_scale: transform.scale.x,
                });

                let material = if maybe_active.is_some() {
                    game_materials.cyan.clone_weak()
                } else {
                    game_materials.orange.clone_weak()
                };
                colour_events.send(NodeColourReq(entity, material));
            }
        }
    });
}

pub fn hover_ended(
    mut commands: Commands,
    mut out_events: EventReader<Pointer<Out>>,
    mut colour_events: EventWriter<NodeColourReq>,
    query_nodes: Query<(Entity, Option<&NodeActive>, Option<&Hovered>)>,
    game_materials: Res<GameMaterials>,
) {
    out_events.read().for_each(|ev| {
        if let Ok((entity, maybe_active, maybe_hovered)) = query_nodes.get(ev.target) {
            if maybe_hovered.is_some() {
                commands.entity(entity).remove::<Hovered>();

                let material = if maybe_active.is_some() {
                    game_materials.node_activated.clone_weak()
                } else {
                    game_materials.node_base.clone_weak()
                };
                colour_events.send(NodeColourReq(entity, material));
            }
        }
    });
}
