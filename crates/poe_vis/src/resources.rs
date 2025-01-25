use bevy::{prelude::*, utils::HashMap};
use poe_tree::{character::Character, type_wrappings::*};

#[derive(Resource)]
pub struct NodeScaling {
    pub min_scale: f32,
    pub max_scale: f32,
    pub base_radius: f32,
    pub hover_multiplier: f32,
    pub hover_fade_time: f32,
}

#[derive(Resource, Deref, DerefMut)]
pub struct ActiveCharacter {
    pub character: poe_tree::character::Character,
}

#[derive(Resource, DerefMut, Deref)]
pub struct RootNode(pub Option<NodeId>);

#[derive(Debug, serde::Deserialize, Default, Resource)]
pub struct UserConfig {
    pub colors: HashMap<String, String>,
    pub controls: HashMap<String, Vec<String>>,

    #[serde(skip_deserializing)]
    #[serde(default)]
    pub character: Character,
}
