//$ src/data/poe_tree/skills.rs
use serde::{Deserialize, Serialize};

use super::stats::{deserialize_stats, Stat};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PassiveSkill {
    pub name: Option<String>,
    #[serde(default)]
    pub is_notable: bool,

    #[serde(default)]
    #[serde(deserialize_with = "deserialize_stats")]
    pub stats: Vec<Stat>,

    //TODO: someday...
    #[serde(skip_deserializing)]
    _ascendanvy: String,
    #[serde(skip_deserializing)]
    _icon: String,
}
