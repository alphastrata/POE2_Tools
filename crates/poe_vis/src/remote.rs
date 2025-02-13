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
    components::{NodeMarker, UIGlyph},
    events::*,
    parse_tailwind_color,
    PassiveTreeWrapper, // resources, materials, etc...
};

pub struct RPCPlugin;

pub enum Command {
    ActivateNode(NodeId),
    DeactivateNode(NodeId),

    ActivateNodeWithColour(NodeId, String),
    ActivateEdgeWthColour(NodeId, NodeId, String),

    ScaleNode(u32, f32),                   // (entity_id, scale)
    GetNodePos(NodeId, Sender<Transform>), // returns the node's position
    GetNodeColour(NodeId),                 // returns the node's colour

    ActivateEdge(EdgeId, EdgeId),
    DeactivateEdge(EdgeId, EdgeId),

    DrawCircle(DrawCircleReq),
    DrawRectangle(DrawRectangleReq),

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

    io.add_sync_method("draw_circle", {
        let tx = tx.clone();
        move |p: Params| {
            let req = parse_draw_circle(&p);
            tx.send(Command::DrawCircle(req)).ok();
            Ok(Value::String("ok".into()))
        }
    });
    io.add_sync_method("draw_rect", {
        let tx = tx.clone();
        move |p: Params| {
            let req = parse_draw_rect(&p);
            tx.send(Command::DrawRectangle(req)).ok();
            Ok(Value::String("ok".into()))
        }
    });

    io.add_sync_method("activate_edge_with_colour", {
        let tx = tx.clone();
        move |p: Params| {
            let (start, end, colour) = parse_edge_with_colour(&p);
            tx.send(Command::ActivateEdgeWthColour(start, end, colour))
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
        Ok(serde_json::Value::Array(
            crate::consts::TAILWIND_COLOURS_AS_STR
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
    // mut edge_col: EventWriter<EdgeColourReq>,
    // mut load: EventWriter<LoadCharacterReq>,
    mut activate_edge_with_colour: EventWriter<ManualEdgeHighlightWithColour>,
    mut activate_node_with_colour: EventWriter<ManualNodeHighlightWithColour>,
    mut activation: EventWriter<NodeActivationReq>,
    mut cam: EventWriter<MoveCameraReq>,
    mut clear: EventWriter<ClearAll>,
    mut deactivation: EventWriter<NodeDeactivationReq>,
    mut draw_circle: EventWriter<DrawCircleReq>,
    mut draw_rect: EventWriter<DrawRectangleReq>,
    mut edge_act: EventWriter<EdgeActivationReq>,
    mut edge_deact: EventWriter<EdgeDeactivationReq>,
    mut scale: EventWriter<NodeScaleReq>,
    // mut node_col: EventWriter<NodeColourReq>,
    // mut save: EventWriter<SaveCharacterReq>,
    tree: Res<PassiveTreeWrapper>, // TODO: additional resources?
    node_positions: Query<(&Transform, &NodeMarker)>,
) {
    while let Ok(cmd) = server.rx.try_recv() {
        match cmd {
            Command::DrawCircle(ev) => {
                draw_circle.send(ev);
            }
            Command::DrawRectangle(ev) => {
                draw_rect.send(ev);
            }
            Command::ClearAll => {
                clear.send(ClearAll);
            }
            Command::ActivateNodeWithColour(id, col) => {
                activate_node_with_colour.send(ManualNodeHighlightWithColour(id, col));
            }
            Command::ActivateEdgeWthColour(start, end, col) => {
                activate_edge_with_colour.send(ManualEdgeHighlightWithColour(start, end, col));
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
                node_positions.par_iter().for_each(|(tf, nid)| {
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

fn parse_edge_with_colour(params: &Params) -> (NodeId, NodeId, String) {
    match params {
        Params::Array(arr) if arr.len() >= 3 => (
            arr[0].as_u64().unwrap() as NodeId,
            arr[1].as_u64().unwrap() as NodeId,
            arr[2].as_str().unwrap().to_string(),
        ),
        _ => unimplemented!(),
    }
}

fn parse_origin(val: &Value) -> Vec3 {
    let arr = val.as_array().unwrap();
    Vec3::new(
        arr[0].as_f64().unwrap() as f32,
        arr[1].as_f64().unwrap() as f32,
        arr[2].as_f64().unwrap() as f32 + 10.0, // Always infront
    )
}

fn parse_material(val: Option<&Value>) -> String {
    //TODO We have the tailwind colours here so we should vaildate against them...
    val.and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_default()
}

fn parse_draw_circle(params: &Params) -> DrawCircleReq {
    match params {
        Params::Array(arr) if arr.len() >= 3 => {
            let radius = arr[0].as_f64().unwrap() as f32;
            let origin = parse_origin(&arr[1]);
            let mat = parse_material(arr.get(2));
            DrawCircleReq {
                radius,
                origin,
                mat,
                glyph: UIGlyph::default(),
            }
        }
        _ => unimplemented!(),
    }
}

fn parse_draw_rect(params: &Params) -> DrawRectangleReq {
    match params {
        Params::Array(arr) if arr.len() >= 3 => {
            let hs_arr = arr[0].as_array().unwrap();
            let half_size = Vec2::new(
                hs_arr[0].as_f64().unwrap() as f32,
                hs_arr[1].as_f64().unwrap() as f32,
            );
            let origin = parse_origin(&arr[1]);

            let mat = parse_material(arr.get(2));
            DrawRectangleReq {
                half_size,
                origin,
                mat,
                glyph: UIGlyph::default(),
            }
        }
        _ => unimplemented!(),
    }
}
