use bevy::prelude::*;
use crossbeam_channel::{unbounded, Receiver, Sender};
use jsonrpc_core::{IoHandler, Params, Value};
use jsonrpc_http_server::{Server as RpcServer, ServerBuilder};
use poe_tree::type_wrappings::{EdgeId, NodeId};

use crate::{
    events::*,
    // resources, materials, etc...
};

pub struct RPCPlugin;

pub enum Command {
    ActivateNode(NodeId),
    DeactivateNode(NodeId),
    ScaleNode(u32, f32),   // (entity_id, scale)
    GetNodePos(NodeId),    // returns the node's position
    GetNodeColour(NodeId), // returns the node's colour

    ActivateEdge(EdgeId, EdgeId),
    DeactivateEdge(EdgeId, EdgeId),

    LoadCharacter,
    SaveCharacter,

    GetCameraPos,              // returns the current camera position
    MoveCamera(f32, f32, f32), // x,y,z

    GetMaterials,            // returns the known material names
    ColourNode(u32, String), // (entity_id, some_color_string)
    ColourEdge(u32, String), // (entity_id, some_color_string)
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
        .start_http(&"0.0.0.0:90210".parse().unwrap())
        .unwrap();

    commands.insert_resource(Server { handle: server, rx });
}

fn add_rpc_io_methods(tx: Sender<Command>) -> IoHandler {
    let mut io = IoHandler::new();

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

    io.add_sync_method("get_camera_pos", {
        move |_params: Params| {
            // Example: return a dummy camera position
            // Real logic would query from your Bevy camera resource
            tx.send(Command::GetCameraPos).ok();
            Ok(Value::String("ok".into()))
        }
    });

    // io.add_sync_method("get_node_pos", {
    //     move |p: Params| {
    //         let node_id = parse_node_id(&p);

    //         tx.send(Command::GetCameraPos).ok();
    //         Ok(Value::String("ok".into()))

    //         Ok(jsonrpc_core::Value::Array(vec![
    //             node_id.into(),
    //             1.23.into(),
    //             4.56.into(),
    //         ]))
    //     }
    // });

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
    mut deactivation: EventWriter<NodeDeactivationReq>,
    mut scale: EventWriter<NodeScaleReq>,
    node_col: EventWriter<NodeColourReq>,
    edge_col: EventWriter<EdgeColourReq>,
    mut edge_act: EventWriter<EdgeActivationReq>,
    mut edge_deact: EventWriter<EdgeDeactivationReq>,
    load: EventWriter<LoadCharacterReq>,
    save: EventWriter<SaveCharacterReq>,
    mut cam: EventWriter<MoveCameraReq>,
    // TODO: additional resources?
) {
    while let Ok(cmd) = server.rx.try_recv() {
        match cmd {
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
            Command::GetNodePos(_) => {
                todo!();
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

// small parse fns for our Params...
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
