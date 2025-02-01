/// If you want a default fade time, you can store it here or in NodeScaling
pub const DEFAULT_HOVER_FADE_TIME: f32 = 0.760;

/// In the game you can reach level 99 + 24 skillpoints from quests.
pub const MAXIMUM_ACTIVATEABLE_NODES: u8 = 123;

pub const NODE_PLACEMENT_Z_IDX: f32 = 0.0;

pub const EDGE_PLACEMENT_Z_IDX: f32 = -10.0;

/// The length a query MUST be in the searchbox before we allow the searching to begin...
pub const SEARCH_THRESHOLD: usize = 5;

/// Default path to which we'll save a character.
pub const DEFAULT_SAVE_PATH: &str = "data/character.toml";
