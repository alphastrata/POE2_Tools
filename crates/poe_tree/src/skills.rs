use serde::Deserialize;

use crate::stats::Stat;

#[derive(Debug, Default, Clone, Deserialize)]
pub struct PassiveSkill {
    name: Option<String>,
    #[serde(default)]
    is_notable: bool,

    #[serde(skip_deserializing)]
    pub stats: Vec<Stat>,

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
