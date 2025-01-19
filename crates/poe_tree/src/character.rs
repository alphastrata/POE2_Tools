use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Character {
    pub class: CharacterClass,
    pub name: String,
    pub activated_node_ids: HashSet<usize>, // Using HashSet for efficient lookup of node IDs
    pub date_created: DateTime<Utc>,

    // POE2 Relevant fields
    pub level: u32,
   
    // Max value = 24
    pub quest_passive_skills: u8,

     // TODO: add some skill related data
    // pub equipped_items: Vec<Item>,
    // pub skill_gems: Vec<SkillGem>
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum CharacterClass {
    #[default]
    Monk,
    Sorceress,
    Witch,
    Warrior,
    Mercenary,
    Ranger,
}

#[cfg(test)]
mod tests {
use super::*; 
    #[test]
    fn can_parse_character() {
      

    }
}