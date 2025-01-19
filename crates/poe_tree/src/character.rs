use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs, io, path::Path};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct Character {
    pub class: CharacterClass,
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

     pub fn load_from_toml<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        let toml_string = fs::read_to_string(path)?;
        let character: Self = toml::from_str(&toml_string).expect("Failed to deserialize from TOML");
        Ok(character)
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
       
        let loaded_character = Character::load_from_toml(test_file_path).unwrap();
        assert_eq!(loaded_character.name, "jengablox");
        assert_eq!(loaded_character.class, CharacterClass::Monk);
        assert_eq!(loaded_character.activated_node_ids.len(), 0);
         assert_eq!(loaded_character.date_created, DateTime::parse_from_rfc3339("2025-01-13T12:34:56Z").unwrap().with_timezone(&Utc));
        assert_eq!(loaded_character.level, 8);
        assert_eq!(loaded_character.quest_passive_skills, 2);

        // Clean up temp file
         std::fs::remove_file(test_file_path).unwrap();
    }
}