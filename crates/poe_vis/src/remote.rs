#![allow(dead_code, unused_variables)]
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use crossbeam_channel::{unbounded, Receiver, Sender};
use jsonrpc_core::{IoHandler, Params, Value};
use jsonrpc_http_server::{Server as RpcServer, ServerBuilder};
use poe_tree::{
    type_wrappings::{EdgeId, NodeId},
    PassiveTree,
};

use crate::{
    components::NodeMarker,
    events::*,
    PassiveTreeWrapper, // resources, materials, etc...
};

pub struct RPCPlugin;

pub enum Command {
    ActivateNode(NodeId),
    DeactivateNode(NodeId),

    ActivateNodeWithColour(NodeId, String),

    ScaleNode(u32, f32),                   // (entity_id, scale)
    GetNodePos(NodeId, Sender<Transform>), // returns the node's position
    GetNodeColour(NodeId),                 // returns the node's colour

    ActivateEdge(EdgeId, EdgeId),
    DeactivateEdge(EdgeId, EdgeId),

    LoadCharacter,
    SaveCharacter,

    GetCameraPos,              // returns the current camera position
    MoveCamera(f32, f32, f32), // x,y,z

    GetMaterials,            // returns the known material names
    ColourNode(u32, String), // (entity_id, some_color_string)
    ColourEdge(u32, String),

    /// Clears all highlithing effectively resetting the tree (visually.)
    ClearAll,
}

#[derive(Resource)]
pub struct Server {
    pub handle: RpcServer,
    pub rx: Receiver<Command>,
}

impl Plugin for RPCPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, setup_server)
            .add_systems(Update, rx_rpx);
    }
}

fn setup_server(mut commands: Commands) {
    let (tx, rx) = unbounded();

    let io = add_rpc_io_methods(tx);

    let server = ServerBuilder::new(io)
        .start_http(&"0.0.0.0:6004".parse().unwrap())
        .unwrap();

    commands.insert_resource(Server { handle: server, rx });
}

fn add_rpc_io_methods(tx: Sender<Command>) -> IoHandler {
    let mut io = IoHandler::new();

    io.add_sync_method("activate_node_with_colour", {
        let tx = tx.clone();
        move |p: Params| {
            let (node_id, colour) = parse_node_with_colour(&p);
            tx.send(Command::ActivateNodeWithColour(node_id, colour))
                .ok();
            Ok(Value::String("ok".into()))
        }
    });

    io.add_sync_method("clear", {
        let tx = tx.clone();
        tx.send(Command::ClearAll).ok();
        move |_p: Params| Ok(Value::String("ok".into()))
    });

    io.add_sync_method("get_available_colours", |_params: Params| {
        let colours = vec![
            "amber-50",
            "amber-100",
            "amber-200",
            "amber-300",
            "amber-400",
            "amber-500",
            "amber-600",
            "amber-700",
            "amber-800",
            "amber-900",
            "amber-950",
            "blue-50",
            "blue-100",
            "blue-200",
            "blue-300",
            "blue-400",
            "blue-500",
            "blue-600",
            "blue-700",
            "blue-800",
            "blue-900",
            "blue-950",
            "cyan-50",
            "cyan-100",
            "cyan-200",
            "cyan-300",
            "cyan-400",
            "cyan-500",
            "cyan-600",
            "cyan-700",
            "cyan-800",
            "cyan-900",
            "cyan-950",
            "emerald-50",
            "emerald-100",
            "emerald-200",
            "emerald-300",
            "emerald-400",
            "emerald-500",
            "emerald-600",
            "emerald-700",
            "emerald-800",
            "emerald-900",
            "emerald-950",
            "fuchsia-50",
            "fuchsia-100",
            "fuchsia-200",
            "fuchsia-300",
            "fuchsia-400",
            "fuchsia-500",
            "fuchsia-600",
            "fuchsia-700",
            "fuchsia-800",
            "fuchsia-900",
            "fuchsia-950",
            "gray-50",
            "gray-100",
            "gray-200",
            "gray-300",
            "gray-400",
            "gray-500",
            "gray-600",
            "gray-700",
            "gray-800",
            "gray-900",
            "gray-950",
            "green-50",
            "green-100",
            "green-200",
            "green-300",
            "green-400",
            "green-500",
            "green-600",
            "green-700",
            "green-800",
            "green-900",
            "green-950",
            "indigo-50",
            "indigo-100",
            "indigo-200",
            "indigo-300",
            "indigo-400",
            "indigo-500",
            "indigo-600",
            "indigo-700",
            "indigo-800",
            "indigo-900",
            "indigo-950",
            "lime-50",
            "lime-100",
            "lime-200",
            "lime-300",
            "lime-400",
            "lime-500",
            "lime-600",
            "lime-700",
            "lime-800",
            "lime-900",
            "lime-950",
            "neutral-50",
            "neutral-100",
            "neutral-200",
            "neutral-300",
            "neutral-400",
            "neutral-500",
            "neutral-600",
            "neutral-700",
            "neutral-800",
            "neutral-900",
            "neutral-950",
            "orange-50",
            "orange-100",
            "orange-200",
            "orange-300",
            "orange-400",
            "orange-500",
            "orange-600",
            "orange-700",
            "orange-800",
            "orange-900",
            "orange-950",
            "pink-50",
            "pink-100",
            "pink-200",
            "pink-300",
            "pink-400",
            "pink-500",
            "pink-600",
            "pink-700",
            "pink-800",
            "pink-900",
            "pink-950",
            "purple-50",
            "purple-100",
            "purple-200",
            "purple-300",
            "purple-400",
            "purple-500",
            "purple-600",
            "purple-700",
            "purple-800",
            "purple-900",
            "purple-950",
            "red-50",
            "red-100",
            "red-200",
            "red-300",
            "red-400",
            "red-500",
            "red-600",
            "red-700",
            "red-800",
            "red-900",
            "red-950",
            "rose-50",
            "rose-100",
            "rose-200",
            "rose-300",
            "rose-400",
            "rose-500",
            "rose-600",
            "rose-700",
            "rose-800",
            "rose-900",
            "rose-950",
            "sky-50",
            "sky-100",
            "sky-200",
            "sky-300",
            "sky-400",
            "sky-500",
            "sky-600",
            "sky-700",
            "sky-800",
            "sky-900",
            "sky-950",
            "slate-50",
            "slate-100",
            "slate-200",
            "slate-300",
            "slate-400",
            "slate-500",
            "slate-600",
            "slate-700",
            "slate-800",
            "slate-900",
            "slate-950",
            "stone-50",
            "stone-100",
            "stone-200",
            "stone-300",
            "stone-400",
            "stone-500",
            "stone-600",
            "stone-700",
            "stone-800",
            "stone-900",
            "stone-950",
            "teal-50",
            "teal-100",
            "teal-200",
            "teal-300",
            "teal-400",
            "teal-500",
            "teal-600",
            "teal-700",
            "teal-800",
            "teal-900",
            "teal-950",
            "violet-50",
            "violet-100",
            "violet-200",
            "violet-300",
            "violet-400",
            "violet-500",
            "violet-600",
            "violet-700",
            "violet-800",
            "violet-900",
            "violet-950",
            "yellow-50",
            "yellow-100",
            "yellow-200",
            "yellow-300",
            "yellow-400",
            "yellow-500",
            "yellow-600",
            "yellow-700",
            "yellow-800",
            "yellow-900",
            "yellow-950",
            "zinc-50",
            "zinc-100",
            "zinc-200",
            "zinc-300",
            "zinc-400",
            "zinc-500",
            "zinc-600",
            "zinc-700",
            "zinc-800",
            "zinc-900",
            "zinc-950",
        ];
        Ok(serde_json::Value::Array(
            colours
                .into_iter()
                .map(|c| serde_json::Value::String(c.to_string()))
                .collect(),
        ))
    });

    io.add_sync_method("activate_node", {
        let tx = tx.clone();
        move |p: Params| {
            tx.send(Command::ActivateNode(parse_node_id(&p))).ok();
            Ok(Value::String("ok".into()))
        }
    });

    io.add_sync_method("deactivate_node", {
        let tx = tx.clone();
        move |p: Params| {
            tx.send(Command::DeactivateNode(parse_node_id(&p))).ok();
            Ok(Value::String("ok".into()))
        }
    });

    io.add_sync_method("scale_node", {
        let tx = tx.clone();
        move |p: Params| {
            let (ent, scale) = parse_node_scale(&p);
            tx.send(Command::ScaleNode(ent, scale)).ok();
            Ok(Value::String("ok".into()))
        }
    });

    io.add_sync_method("colour_node", {
        let tx = tx.clone();
        move |p: Params| {
            let (ent, colour) = parse_node_colour(&p);
            tx.send(Command::ColourNode(ent, colour)).ok();
            Ok(Value::String("ok".into()))
        }
    });

    io.add_sync_method("colour_edge", {
        let tx = tx.clone();
        move |p: Params| {
            let (ent, colour) = parse_node_colour(&p);
            tx.send(Command::ColourEdge(ent, colour)).ok();
            Ok(Value::String("ok".into()))
        }
    });

    io.add_sync_method("activate_edge", {
        let tx = tx.clone();
        move |p: Params| {
            let (e1, e2) = parse_edge_ids(&p);
            tx.send(Command::ActivateEdge(e1, e2)).ok();
            Ok(Value::String("ok".into()))
        }
    });

    io.add_sync_method("deactivate_edge", {
        let tx = tx.clone();
        move |p: Params| {
            let (e1, e2) = parse_edge_ids(&p);
            tx.send(Command::DeactivateEdge(e1, e2)).ok();
            Ok(Value::String("ok".into()))
        }
    });

    //TODO: Need to take a Path
    io.add_sync_method("load_character", {
        let tx = tx.clone();
        move |_p: Params| {
            tx.send(Command::LoadCharacter).ok();
            Ok(Value::String("ok".into()))
        }
    });

    io.add_sync_method("save_character", {
        let tx = tx.clone();
        move |_p: Params| {
            tx.send(Command::SaveCharacter).ok();
            Ok(Value::String("ok".into()))
        }
    });

    io.add_sync_method("move_camera", {
        let tx = tx.clone();
        move |p: Params| {
            let (x, y, z) = parse_vec3(&p);
            tx.send(Command::MoveCamera(x, y, z)).ok();
            Ok(Value::String("ok".into()))
        }
    });

    io.add_sync_method("ping", |_p: Params| Ok(Value::String("ok".into())));

    // TODO: will need a way to query world...
    // io.add_sync_method("get_materials", {
    //     move |_params: Params| {
    //         // example: just return all known fields from GameMaterials
    //         let mats = vec![
    //             "node_base","node_attack","node_mana","node_dexterity","node_intelligence",
    //             "node_strength","node_activated","edge_base","edge_activated","background",
    //             "foreground","red","orange","yellow","green","blue","purple","cyan",
    //         ];
    //         // return a JSON array of these
    //         Ok(jsonrpc_core::Value::Array(
    //             mats.into_iter().map(jsonrpc_core::Value::String).collect()
    //         ))
    //     }
    // });

    // io.add_sync_method("get_camera_pos", {
    //     move |_params: Params| {
    //         // Example: return a dummy camera position
    //         // Real logic would query from your Bevy camera resource
    //         tx.send(Command::GetCameraPos).ok();
    //         Ok(Value::String("ok".into()))
    //     }
    // });

    io.add_sync_method("get_node_pos", {
        move |p: Params| {
            let node_id = parse_node_id(&p);
            let (sender, receiver) = crossbeam_channel::bounded(1);
            tx.send(Command::GetNodePos(node_id, sender)).ok();

            // Wait for the Transform response.
            let tf: Transform = receiver.recv().unwrap();
            let pos = tf.translation;

            // Return JSON array [x, y, z]
            Ok(Value::Array(vec![
                Value::Number(serde_json::Number::from_f64(pos.x as f64).unwrap()),
                Value::Number(serde_json::Number::from_f64(pos.y as f64).unwrap()),
                Value::Number(serde_json::Number::from_f64(pos.z as f64).unwrap()),
            ]))
        }
    });

    // io.add_sync_method("get_node_colour", {
    //     move |p: Params| {
    //         let node_id = parse_node_id(&p);
    //         // Example: return a dummy colour
    //         // Real logic would map your node to its current material
    //         Ok(jsonrpc_core::Value::String(format!(
    //             "Node {node_id} is red"
    //         )))
    //     }
    // });

    io
}

fn rx_rpx(
    server: Res<Server>,
    mut activation: EventWriter<NodeActivationReq>,
    mut activate_with_colour: EventWriter<ManualHighlightWithColour>,
    mut deactivation: EventWriter<NodeDeactivationReq>,
    mut clear: EventWriter<ClearAll>,
    mut scale: EventWriter<NodeScaleReq>,
    node_col: EventWriter<NodeColourReq>,
    edge_col: EventWriter<EdgeColourReq>,
    mut edge_act: EventWriter<EdgeActivationReq>,
    mut edge_deact: EventWriter<EdgeDeactivationReq>,
    load: EventWriter<LoadCharacterReq>,
    save: EventWriter<SaveCharacterReq>,
    mut cam: EventWriter<MoveCameraReq>,
    tree: Res<PassiveTreeWrapper>, // TODO: additional resources?
    node_positions: Query<(&Transform, &NodeMarker)>,
) {
    while let Ok(cmd) = server.rx.try_recv() {
        match cmd {
            Command::ClearAll => {
                clear.send(ClearAll);
            }
            Command::ActivateNodeWithColour(id, col) => {
                activate_with_colour.send(ManualHighlightWithColour(id, col));
            }
            Command::ActivateNode(id) => {
                activation.send(NodeActivationReq(id));
            }
            Command::DeactivateNode(id) => {
                deactivation.send(NodeDeactivationReq(id));
            }
            Command::ScaleNode(ent, s) => {
                scale.send(NodeScaleReq(Entity::from_raw(ent), s));
            }
            Command::ActivateEdge(e1, e2) => {
                edge_act.send(EdgeActivationReq(e1, e2));
            }
            Command::DeactivateEdge(e1, e2) => {
                edge_deact.send(EdgeDeactivationReq(e1, e2));
            }
            Command::MoveCamera(x, y, z) => {
                cam.send(MoveCameraReq(Vec3::new(x, y, z)));
            }
            Command::GetNodePos(target, oneshot) => {
                let mut m_tf = Transform::default();
                let mtx_tf = Arc::new(Mutex::new(&mut m_tf));
                let loc = node_positions.par_iter().for_each(|(tf, nid)| {
                    if **nid == target {
                        if let Err(e) = oneshot.send(*tf) {
                            log::error!(
                                "Unable to send the transform for the node {}'s position.\nError msg: {e}",
                                target
                            );
                        }
                    }
                });
            }
            Command::GetNodeColour(_) => {
                todo!();
            }
            Command::LoadCharacter => {
                todo!();
            }
            Command::SaveCharacter => {
                todo!();
            }
            Command::GetCameraPos => {
                todo!();
            }
            Command::GetMaterials => {
                todo!();
            }
            Command::ColourNode(_, _) => {
                todo!();
            }
            Command::ColourEdge(_, _) => {
                todo!();
            }
        }
    }
}

// --------------- Parsing helpers --------------- //
fn parse_node_id(params: &Params) -> NodeId {
    match params {
        Params::Array(arr) if !arr.is_empty() => arr[0].as_u64().unwrap() as NodeId,
        _ => unimplemented!(),
    }
}
fn parse_node_scale(params: &Params) -> (u32, f32) {
    // e.g. [1234, 2.5]
    match params {
        Params::Array(arr) if arr.len() >= 2 => {
            let ent = arr[0].as_u64().unwrap() as u32;
            let scl = arr[1].as_f64().unwrap() as f32;
            (ent, scl)
        }
        _ => unimplemented!(),
    }
}
fn parse_node_colour(params: &Params) -> (u32, String) {
    // e.g. [1234, "red"]
    match params {
        Params::Array(arr) if arr.len() >= 2 => {
            let ent = arr[0].as_u64().unwrap() as u32;
            let col = arr[1].as_str().unwrap().to_string();
            (ent, col)
        }
        _ => unimplemented!(),
    }
}
fn parse_edge_ids(params: &Params) -> (EdgeId, EdgeId) {
    // e.g. [55, 66]
    match params {
        Params::Array(arr) if arr.len() >= 2 => (
            arr[0].as_u64().unwrap() as EdgeId,
            arr[1].as_u64().unwrap() as EdgeId,
        ),
        _ => unimplemented!(),
    }
}
fn parse_vec3(params: &Params) -> (f32, f32, f32) {
    // e.g. [1.0, 2.0, 3.0]
    match params {
        Params::Array(arr) if arr.len() >= 3 => {
            let x = arr[0].as_f64().unwrap() as f32;
            let y = arr[1].as_f64().unwrap() as f32;
            let z = arr[2].as_f64().unwrap() as f32;
            (x, y, z)
        }
        _ => unimplemented!(),
    }
}

fn parse_node_with_colour(params: &Params) -> (NodeId, String) {
    match params {
        Params::Array(arr) if arr.len() >= 2 => (
            arr[0].as_u64().unwrap() as NodeId,
            arr[1].as_str().unwrap().to_string(),
        ),
        _ => unimplemented!(),
    }
}
