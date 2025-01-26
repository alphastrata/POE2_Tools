//! The place for all things that allow poe_vis to act as server,
//! taking requests via RPC etc to make shit happen.
//!
use bevy::prelude::*;
use jsonrpc_core::{IoHandler, Params, Value};
use jsonrpc_http_server::{Server as RpcServer, ServerBuilder};
use poe_tree::type_wrappings::NodeId;
use crossbeam::{unvounded, Receiver, Sender};

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
    // create channel
    let (tx, rx): (Sender<Command>, Receiver<Command>) = unbounded();

    // build rpc handler
    let mut io = IoHandler::new();

    // Add more methods here:
    io.add_method("activate_node", move |params: Params| {
        // parse node id
        let node_id: NodeId = parse_node_id(&params); 
        tx.send(Command::ActivateNode(node_id)).ok();
        Ok(Value::String("ok".into()))
    });
    // add other methods similarly...

    // start server
    let server = ServerBuilder::new(io)
        .start_http(&"0.0.0.0:90210".parse().unwrap())
        .unwrap();

    // store resource
    commands.insert_resource(Server { handle: server, rx });
}

fn rx_rpx(
    server: Res<Server>, 
    mut activate: EventWriter<NodeActivationReq>,
    mut deactivate: EventWriter<NodeDeactivationReq>,
    // etc for highlight, remove highlight...
) {
    // pump commands into events
    while let Ok(cmd) = server.rx.try_recv() {
        match cmd {
            Command::ActivateNode(id) => activate.send(NodeActivationReq(id)),
            Command::DeactivateNode(id) => deactivate.send(NodeDeactivationReq(id)),
            _=>{}
        }
    }
}

// Example parser
fn parse_node_id(params: &Params) -> NodeId {
    match params {
        Params::Array(arr) if !arr.is_empty() => {
            let val = arr[0].as_u64().unwrap_or(0) as usize;
            NodeId(val)
        },
        _ => NodeId(0),
    }
}

// Example curl test (activate_node):
// curl -X POST -H "Content-Type: application/json" \
//   --data '{"jsonrpc":"2.0","method":"activate_node","params":[123],"id":1}' \
//   http://0.0.0.0:90210
