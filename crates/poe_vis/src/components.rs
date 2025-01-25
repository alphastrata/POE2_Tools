use bevy::prelude::Component;
use poe_tree::type_wrappings::{GroupId, NodeId};

#[derive(Component)]
pub struct NodeMarker(pub NodeId); // Marker component for nodes

#[derive(Component)]
pub struct EdgeMarker(pub (NodeId, NodeId)); // Marker component for nodes

#[derive(Component)]
pub struct NodeActive;

#[derive(Component)]
pub struct EdgeActive;

#[derive(Component)]
pub struct NodeInactive;

#[derive(Component)]
pub struct EdgeInactive;

#[derive(Component)]
pub struct GroupMarker(pub GroupId);


#[derive(Component)]
pub struct NodeHoverText;

#[derive(Component)]
pub struct Hovered {
    pub timer: bevy::time::Timer,
    pub base_scale: f32,
}
