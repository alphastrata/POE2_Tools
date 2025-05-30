use std::time::Duration;

use bevy::{
    prelude::{Component, Deref, DerefMut},
    time::Timer,
};
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

#[derive(Component, DerefMut, Deref, Clone, Debug)]
pub struct UIGlyph(pub Timer);
impl UIGlyph {
    pub fn set(&mut self, duration: Duration) {
        self.0 = Timer::new(duration, bevy::time::TimerMode::Once);
    }

    pub fn new_with_duration(duration: f32) -> Self {
        Self(Timer::from_seconds(duration, bevy::time::TimerMode::Once))
    }

    pub fn from_millis(millis: u64) -> Self {
        let duration = Duration::from_millis(millis);
        Self(Timer::new(duration, bevy::time::TimerMode::Once))
    }
}

impl Default for UIGlyph {
    fn default() -> Self {
        Self(Timer::from_seconds(1.5, bevy::time::TimerMode::Once))
    }
}
