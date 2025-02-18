use bevy::{prelude::*, render::render_resource::ShaderType};
use std::collections::VecDeque;

use crate::{
    components::*,
    consts::DEFAULT_HOVER_FADE_TIME,
    events::{
        ClearVirtualPath, NodeActivationReq, NodeColourReq, NodeDeactivationReq, VirtualPathReq,
    },
    materials::GameMaterials,
    resources::*,
};

#[derive(ShaderType, Debug, Clone)]
pub struct MousePos {
    pub x: f32,
    pub y: f32,
}

pub struct MouseControlsPlugin;

impl Plugin for MouseControlsPlugin {
    fn build(&self, app: &mut App) {
        app
            //
            .insert_resource(MouseSelecetedNodeHistory(VecDeque::new()))
            // .insert_resource(resource)
            //
            ;

        // app.add_systems(
        //     Update,
        //     insert_root_to_history.run_if(RootNode::is_set.or(resource_changed::<ActiveCharacter>)),
        // );

        app.add_systems(
            Update,
            (hover_ticker, handle_node_clicks, hover_started, hover_ended),
        );
    }
}
fn insert_root_to_history(
    root: Res<crate::resources::RootNode>,
    mut last_ten: ResMut<MouseSelecetedNodeHistory>,
) {
    last_ten.clear();
    last_ten.push_back(root.0.expect("Protected by run conditions."));
    log::debug!("Root inserted into mouse's click history");
}

pub fn hover_ticker(time: Res<Time>, mut query_hovz: Query<(Entity, &mut Hovered)>) {
    query_hovz.iter_mut().for_each(|(_ent, mut hov)| {
        hov.timer.tick(time.delta());
    })
}

pub fn handle_node_clicks(
    root: Res<crate::resources::RootNode>,
    mut last_ten: ResMut<MouseSelecetedNodeHistory>,
    mut click_events: EventReader<Pointer<Down>>,
    nodes_query: Query<
        (
            Entity,
            &NodeMarker,
            Option<&NodeInactive>,
            Option<&NodeActive>,
        ),
        // Or<(With<NodeInactive>, With<NodeActive>)>,
    >,
    mut path_repair: ResMut<PathRepairRequired>,
    mut activate_events: EventWriter<NodeActivationReq>,
    mut deactivate_events: EventWriter<NodeDeactivationReq>,
) {
    click_events.read().for_each(|ptr_down| {
        //
        match ptr_down.button {
            PointerButton::Primary => {
                if let Ok((_entity, marker, inactive, active)) = nodes_query.get(ptr_down.target) {
                    match (inactive, active) {
                        // node is inactive -> activate
                        (Some(_), None) => {
                            if root.0.is_some() {
                                activate_events.send(NodeActivationReq(marker.0));
                            }
                            path_repair.request_path_repair();
                        }
                        // node is active -> deactivate
                        (None, Some(_)) => {
                            deactivate_events.send(NodeDeactivationReq(marker.0));
                            path_repair.request_path_repair();
                        }
                        _ => {
                            log::warn!("Should be unreachable...");
                            return;
                        }
                    }
                    last_ten.push_back(marker.0);
                }
            }
            PointerButton::Secondary => {
                log::info!("Right Click interactions are a TODO!")
            }
            PointerButton::Middle => {
                // used to drag camera, explicitly do nothing.
            }
        };
    });
}

pub fn hover_started(
    mut commands: Commands,
    mut over_events: EventReader<Pointer<Over>>,
    mut colour_events: EventWriter<NodeColourReq>,
    mut virt_path_starter: EventWriter<VirtualPathReq>,
    mut clear_virt_path: EventWriter<ClearVirtualPath>,
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
        if let Ok((entity, nm, transform, maybe_hovered, maybe_active)) = query_nodes.get(ev.target)
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
                virt_path_starter.send(VirtualPathReq(**nm));
            }
        }
        // we didn't pikup a hover.
        else {
            clear_virt_path.send(ClearVirtualPath);
        }
    });
}

pub fn hover_ended(
    mut commands: Commands,
    mut out_events: EventReader<Pointer<Out>>,
    mut colour_events: EventWriter<NodeColourReq>,
    mut virt_path_clear: EventWriter<ClearVirtualPath>,
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
                virt_path_clear.send(ClearVirtualPath);
            }
        }
    });
}
