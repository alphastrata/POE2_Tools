use serde::{Deserialize, Serialize};

//$ src/data/poe_tree/skills.rs
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PassiveSkill {
    pub name: Option<String>,
    pub is_notable: bool,
    pub stats: Vec<super::stats::Stat>,
}
