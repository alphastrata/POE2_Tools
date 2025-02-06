//!
//! Events are usually responded to by the background_services.rs code, so thats where they're all added.
//!
use bevy::prelude::Deref;
use bevy::prelude::*;
use poe_tree::type_wrappings::*;
use std::path::PathBuf;

#[derive(Event, DerefMut, Deref)]
pub struct VirtualPathReq(pub NodeId);

#[derive(Event)]
pub struct ClearVirtualPath;

#[derive(Event)]
pub struct NodeScaleReq(pub Entity, pub f32);

#[derive(Event)]
pub struct NodeColourReq(pub Entity, pub Handle<ColorMaterial>);

/// Like [`NodeColourReq`], but _this_ takes a NodeId and any string from the tailwind colours.
#[derive(Event)]
pub struct ManualHighlightWithColour(pub NodeId, pub String);

#[derive(Event)]
pub struct EdgeColourReq(pub Entity, pub Handle<ColorMaterial>);

#[derive(Event, Deref)]
pub struct NodeActivationReq(pub NodeId);

#[derive(Event, Deref)]
pub struct NodeDeactivationReq(pub NodeId);

#[derive(Event, Deref, DerefMut)]
pub struct LoadCharacterReq(pub PathBuf);

#[derive(Event)]
pub struct SaveCharacterReq;

#[derive(Event, Deref)]
pub struct MoveCameraReq(pub Vec3);

#[derive(Event)]
pub struct ClearAll;

#[derive(Event)]
pub struct EdgeActivationReq(pub EdgeId, pub EdgeId);
impl EdgeActivationReq {
    pub(crate) fn as_tuple(&self) -> (EdgeId, EdgeId) {
        (self.0, self.1)
    }
}

#[derive(Event)]
pub struct EdgeDeactivationReq(pub EdgeId, pub EdgeId);
impl EdgeDeactivationReq {
    pub(crate) fn as_tuple(&self) -> (EdgeId, EdgeId) {
        (self.0, self.1)
    }
}

#[derive(Event)]
pub struct ShowSearch;

#[derive(Event)]
pub struct ThrowWarning(String);
