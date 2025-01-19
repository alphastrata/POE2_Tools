//$ crates/poe_tree/src/data/poe_tree/character.rs
#[derive(Debug, serde::Deserialize, Default)]
pub struct CharacterConfig {
    pub class: CharacterClass, // The actual enum field
}

#[derive(Debug, Default)]
pub enum CharacterClass {
    #[default]
    Monk,
    Sorceress,
    Witch,
    Warrior,
    Mercenary,
    Ranger,
}

impl<'de> serde::Deserialize<'de> for CharacterClass {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "Monk" => Ok(CharacterClass::Monk),
            "Sorceress" => Ok(CharacterClass::Sorceress),
            "Witch" => Ok(CharacterClass::Witch),
            "Warrior" => Ok(CharacterClass::Warrior),
            "Mercenary" => Ok(CharacterClass::Mercenary),
            "Ranger" => Ok(CharacterClass::Ranger),
            _ => Err(serde::de::Error::unknown_variant(
                &s,
                &[
                    "Monk",
                    "Sorceress",
                    "Witch",
                    "Warrior",
                    "Mercenary",
                    "Ranger",
                ],
            )),
        }
    }
}


mod tests{
    
    #[test]
    fn can_parse_config() {
        use crate::config::UserConfig;
        let _config: UserConfig = UserConfig::load_from_file("/Users/smak/Documents/poo-tools2/data/user_config.toml");
    }
}