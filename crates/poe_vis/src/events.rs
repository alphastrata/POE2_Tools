// Try to keep the events organised!

use bevy::prelude::*;
use std::path::PathBuf;

use crate::components::UIGlyph;
use poe_tree::type_wrappings::*;

//----------- OPTIMISATION -------------- //
#[derive(Event)]
pub struct OptimiseReq {
    pub selector: Box<dyn Fn(&poe_tree::stats::Stat) -> bool + Send + Sync>,
    pub delta: usize,
}

//----------- VIRTUAL PATHS -------------- //
#[derive(Event, DerefMut, Deref)]
pub struct VirtualPathReq(pub NodeId);

#[derive(Event)]
pub struct ClearVirtualPath;

#[derive(Event)]
pub struct ClearVirtualPaths;

//----------- SEARCH -------------- //
#[derive(Event)]
pub struct ClearSearchResults;

#[derive(Event)]
pub struct ShowSearch;

//----------- EDGES -------------- //
#[derive(Event)]
pub struct ManualEdgeHighlightWithColour(pub NodeId, pub NodeId, pub String);

#[derive(Event)]
pub struct EdgeColourReq(pub Entity, pub Handle<ColorMaterial>);

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

//----------- NODES-------------- //
#[derive(Event, Deref)]
pub struct NodeActivationReq(pub NodeId);

#[derive(Event, Deref)]
pub struct NodeDeactivationReq(pub NodeId);

#[derive(Event)]
pub struct NodeScaleReq(pub Entity, pub f32);

#[derive(Event)]
pub struct NodeColourReq(pub Entity, pub Handle<ColorMaterial>);

#[derive(Event)]
pub struct ManualNodeHighlightWithColourReq(pub NodeId, pub String);

#[derive(Event)]
pub struct OverrideRootNodeReq(pub NodeId);

//----------- CHARACTER -------------- //
#[derive(Event, Deref, DerefMut)]
pub struct LoadCharacterReq(pub PathBuf);

#[derive(Event)]
pub struct SyncCharacterReq();

/// NOTE: this will invoke a ClearAll as it'd be stupid to preserve a path when someone's chaging class...
/// NOTE: this will change the root node.
#[derive(Event)]
pub struct OverrideCharacterClassReq(pub poe_tree::character::CharacterClass);

#[derive(Event)]
pub struct SaveCharacterReq;

#[derive(Event, Deref, DerefMut)]
pub struct OverrideCharacterNodesReq(pub Vec<NodeId>);

#[derive(Event, Debug, Deref, DerefMut)]
pub struct SaveCharacterAsReq(pub PathBuf);

//----------- CAMERA -------------- //
#[derive(Event, Deref)]
pub struct MoveCameraReq(pub Vec3);

//----------- GLOBAL CLEAR -------------- //
#[derive(Event)]
pub struct ClearAllReqReq;

//----------- WARNINGS -------------- //
#[derive(Event)]
pub struct ThrowWarning(String);

//----------- DRAG AND DROP -------------- //
#[derive(Event, Debug, Deref, DerefMut)]
pub struct DragNDrop {
    pub path: PathBuf,
}

//----------- DRAWING -------------- //
#[derive(Event)]
pub struct DrawRectangleReq {
    pub half_size: Vec2,
    pub origin: Vec3,
    pub mat: String,
    pub glyph: UIGlyph,
}

#[derive(Event)]
pub struct DrawCircleReq {
    pub radius: f32,
    pub origin: Vec3,
    pub mat: String,
    pub glyph: UIGlyph,
}
