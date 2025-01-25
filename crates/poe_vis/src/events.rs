// events.rs
use bevy::prelude::*;
use poe_tree::type_wrappings::*;

#[derive(Event)]
pub struct ScaleNode(pub Entity, pub f32);

#[derive(Event)]
pub struct ColourNode(pub Entity, pub Handle<ColorMaterial>);

#[derive(Event)]
pub struct ActivateNode(NodeId);

#[derive(Event)]
pub struct ActivateEdge(EdgeId);

#[derive(Event)]
pub struct DeactivateNode(NodeId);

#[derive(Event)]
pub struct DeactivateEdge(EdgeId);

pub struct LoadCharacter;
pub struct SaveCharacter;

#[derive(Event)]
pub struct MoveCameraReq(Vec3);
