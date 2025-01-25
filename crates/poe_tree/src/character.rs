//!$ crates/poe_tree/src/character.rs
use crate::type_wrappings::NodeId;
use chrono::{DateTime, Utc};
use core::fmt;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs, io,
    path::Path,
};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Character {
    pub character_class: CharacterClass,
    pub name: String,
    pub activated_node_ids: HashSet<NodeId>,
    pub date_created: DateTime<Utc>,
    pub level: u32,
    pub quest_passive_skills: u8,
    pub starting_node: NodeId,
}

impl Character {
    pub fn save_to_toml<P: AsRef<Path>>(&self, path: P) -> Result<(), io::Error> {
        let toml_string = toml::to_string(self).expect("Failed to serialize to TOML");
        fs::write(path, toml_string)
    }

    pub fn load_from_toml<P: AsRef<Path>>(path: P) -> Option<Self> {
        let toml_string = fs::read_to_string(path.as_ref()).unwrap_or_default();

        toml::from_str(&toml_string).ok().or_else(|| {
            eprintln!(
                "Failed to deserialize from TOML from {}. DATA:\n{}",
                path.as_ref().display(),
                toml_string,
            );
            None
        })
    }
}

impl fmt::Display for Character {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Class: {}\nName: {}\nActivated Nodes: {}\nDate Created: {}\nLevel: {}\nQuest Passive Skills: {}",
            self.character_class,
            self.name,
            self.activated_node_ids.len(),
            self.date_created,
            self.level,
            self.quest_passive_skills,
        )
    }
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub enum CharacterClass {
    #[default]
    Monk,
    Sorceress,
    Witch,
    Warrior,
    Mercenary,
    Ranger,
}

impl fmt::Display for CharacterClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let class_name = match self {
            CharacterClass::Monk => "Monk",
            CharacterClass::Sorceress => "Sorceress",
            CharacterClass::Witch => "Witch",
            CharacterClass::Warrior => "Warrior",
            CharacterClass::Mercenary => "Mercenary",
            CharacterClass::Ranger => "Ranger",
        };
        write!(f, "{}", class_name)
    }
}

impl CharacterClass {
    pub fn to_string(&self) -> &str {
        match self {
            CharacterClass::Monk => "Monk",
            CharacterClass::Sorceress => "Sorceress",
            CharacterClass::Witch => "Witch",
            CharacterClass::Warrior => "Warrior",
            CharacterClass::Mercenary => "Mercenary",
            CharacterClass::Ranger => "Ranger",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn load_from_toml_file_example() {
        let test_file_path = "../../data/character.toml";

        _ = Character::load_from_toml(test_file_path).unwrap();
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CharacterStats {
    // Basic Info
    pub level: u8,
    pub character_class: CharacterClass,
    pub name: String,

    // Attributes
    pub strength: u32,
    pub dexterity: u32,
    pub intelligence: u32,

    // Primary Stats
    pub life: u32,
    pub energy_shield: u32,
    pub mana: u32,
    pub spirit: u32,

    // Defensive Stats (raw values, not percentages)
    pub armour: u32,
    pub evasion: u32,
    pub block: u32,

    // Resistances (current and capped values), it's not possible to get more than 255 right?!
    pub fire_resistance: u8,
    pub cold_resistance: u8,
    pub lightning_resistance: u8,
    pub chaos_resistance: u8,
    pub resistance_cap: u8,

    // Derived Stats
    pub evasion_rating: u32,
    pub estimated_chance_to_evade: f32,

    pub mana_recovery_per_second: f32,
    pub mana_recovery_from_regeneration: f32,

    // Difficutly?
    pub difficutly: Difficulty,

    // Miscellaneous Stats
    pub misc: HashMap<String, String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum Difficulty {
    #[default]
    Normal,
    Cruel,
}
impl CharacterStats {
    pub fn new() -> Self {
        Self {
            resistance_cap: 75,
            ..Default::default()
        }
    }

    /// Calculate effective resistance after applying the cap
    pub fn effective_resistance(&self, resistance: u8) -> u8 {
        resistance.min(self.resistance_cap)
    }

    /// Display resistances with capped values
    pub fn display_resistances(&self) -> String {
        format!(
            "Fire: {}% (Max {})\nCold: {}% (Max {})\nLightning: {}% (Max {})\nChaos: {}% (Max {})",
            self.effective_resistance(self.fire_resistance),
            self.resistance_cap,
            self.effective_resistance(self.cold_resistance),
            self.resistance_cap,
            self.effective_resistance(self.lightning_resistance),
            self.resistance_cap,
            self.effective_resistance(self.chaos_resistance),
            self.resistance_cap,
        )
    }

    /// Create a default Monk character based on provided context
    pub fn default_monk() -> Self {
        Self {
            level: 1,
            character_class: CharacterClass::Monk,
            name: String::from("Default Monk"),
            strength: 7,
            dexterity: 11,
            intelligence: 11,
            life: 42,
            mana: 56,
            evasion: 30,
            resistance_cap: 75,
            evasion_rating: 30,
            estimated_chance_to_evade: 6.0,
            mana_recovery_per_second: 2.2,
            mana_recovery_from_regeneration: 2.2,
            misc: HashMap::new(),
            ..Default::default()
        }
    }
}
