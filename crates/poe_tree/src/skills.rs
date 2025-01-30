//!$ crates/poe_tree/src/skills.rs
use serde::{Deserialize, Serialize};

use super::stats::{deserialize_stats, Stat};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PassiveSkill {
    name: Option<String>,
    #[serde(default)]
    is_notable: bool,

    #[serde(default)]
    #[serde(deserialize_with = "deserialize_stats")]
    stats: Vec<Stat>,

    //TODO: someday...
    #[serde(skip_deserializing)]
    _ascendanvy: String,
    #[serde(skip_deserializing)]
    _icon: String,
}

// Beacsue we don't want ppl to 'edit' nodes or their associated Passives.
impl PassiveSkill {
    /// Is this Passive a + to Str/Dex/Int
    pub fn is_attribute(&self) -> bool {
        false
    }

    pub fn is_notable(&self) -> bool {
        self.is_notable
    }
    pub fn stats(&self) -> &[Stat] {
        &self.stats
    }
    /// NOTE PANICS! beware!
    pub fn name(&self) -> String {
        self.name.clone().unwrap()
    }
}
