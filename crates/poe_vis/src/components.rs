use bevy::prelude::Component;
use poe_tree::type_wrappings::{EdgeId, GroupId, NodeId};

#[derive(Component)]
pub struct NodeMarker(pub NodeId); // Marker component for nodes

#[derive(Component)]
pub struct EdgeMarker(pub (EdgeId, EdgeId)); // Marker component for nodes

#[derive(Component)]
pub struct NodeActive(pub bool);

#[derive(Component)]
pub struct EdgeActive(pub bool);

#[derive(Component)]
pub struct GroupMarker(pub GroupId);
