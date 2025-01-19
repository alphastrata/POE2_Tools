use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs, io, path::Path};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Character {
    pub character_class: CharacterClass,
    pub name: String,
    pub activated_node_ids: HashSet<usize>,
    pub date_created: DateTime<Utc>,
    pub level: u32,
    pub quest_passive_skills: u8,
}

impl Character {
    pub fn save_to_toml<P: AsRef<Path>>(&self, path: P) -> Result<(), io::Error> {
        let toml_string = toml::to_string(self).expect("Failed to serialize to TOML");
        fs::write(path, toml_string)
    }
}
impl Character {
    pub fn load_from_toml<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        let toml_string = fs::read_to_string(path.as_ref())?;
        Ok(toml::from_str(&toml_string).expect(&format!(
            "Failed to deserialize from TOML from {}. DATA:\n{}",
            path.as_ref().display(),
            toml_string,
        )))
    }
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
    fn load_from_toml_file_example() {
        let test_file_path = "../../data/character.toml";

        _ = Character::load_from_toml(test_file_path).unwrap();
    }
}
