use bevy::prelude::{Component, Deref, DerefMut};
use poe_tree::{
    skills::PassiveSkill,
    type_wrappings::{GroupId, NodeId},
};

#[derive(Component, Deref, DerefMut)]
pub struct NodeMarker(pub NodeId); // Marker component for nodes

#[derive(Component, Debug, Clone)]
pub struct EdgeMarker(pub NodeId, pub NodeId); // Marker component for nodes

impl EdgeMarker {
    pub(crate) fn as_tuple(&self) -> (NodeId, NodeId) {
        (self.0, self.1)
    }
}

/// Use this to pin an entity which will take it OUT
/// of being influenced by other systems.
#[derive(Component)]
pub struct Pinned;

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

#[derive(Component, Deref)]
pub struct Skill(pub PassiveSkill);

#[derive(Component)]
pub struct SearchMarker;

#[derive(Component)]
pub struct SearchResult;

#[derive(Component)]
pub struct ActiveNodeCount;

#[derive(Component)]
pub struct VirtualPathMember;

#[derive(Component)]
pub struct ManuallyHighlighted;
