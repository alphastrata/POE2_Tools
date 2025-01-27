//! The place for all things that allow poe_vis to act as server,
//! taking requests via RPC etc to make shit happen.
//!
use bevy::prelude::*;
use jsonrpc_core::{IoHandler, Params, Value};
use jsonrpc_http_server::{Server as RpcServer, ServerBuilder};
use poe_tree::type_wrappings::NodeId;
use crossbeam_channel::{unbounded, Receiver, Sender};

use crate::{events::*, materials::{self, GameMaterials}, resources::*};

pub struct RPCPlugin;

// simple enum for commands
pub enum Command {
    ActivateNode(NodeId),
    DeactivateNode(NodeId),
    HighlightNode(NodeId),
    RemoveHighlight(NodeId),
}

#[derive(Resource)]
pub struct Server {
    pub handle: RpcServer,     // the running server
    pub rx: Receiver<Command>, // channel for inbound commands
}

impl Plugin for RPCPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, setup_server).add_systems(Update,rx_rpx);
    }
}

fn setup_server(mut commands: Commands) {
    let (tx, rx): (Sender<Command>, Receiver<Command>) = unbounded();

    let mut io = IoHandler::new();

    // Use add_sync_method to avoid "not a future" error
    io.add_sync_method("activate_node", {
        let tx = tx.clone();
        move |params: Params| {
            let node_id: NodeId = parse(&params);
            tx.send(Command::ActivateNode(node_id)).ok();
            Ok(Value::String("ok".into()))
        }
    });

    io.add_sync_method("deactivate_node", {
        let tx = tx.clone();
        move |params: Params| {
            let node_id: NodeId = parse(&params);
            tx.send(Command::DeactivateNode(node_id)).ok();
            Ok(Value::String("ok".into()))
        }
    });

    // Add more methods like highlight_node, etc...

    let server = ServerBuilder::new(io)
        .start_http(&"0.0.0.0:6004".parse().unwrap())
        .unwrap();

    commands.insert_resource(Server { handle: server, rx });
}

fn rx_rpx(
    server: Res<Server>,
    mut activate: EventWriter<NodeActivationReq>,
    mut deactivate: EventWriter<NodeDeactivationReq>,
    // etc...
) {
    while let Ok(cmd) = server.rx.try_recv() {
        match cmd {
            Command::ActivateNode(id) => {
                activate.send(NodeActivationReq(id));
            }
            Command::DeactivateNode(id) => {
                deactivate.send(NodeDeactivationReq(id));
            }
            _ => {}
        }
    }
}

fn parse(params: &Params) -> NodeId {
    match params {
        Params::Array(arr) if !arr.is_empty() => {
            arr[0].as_u64().unwrap() as NodeId
        }
        _ => unimplemented!()
    }
}