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
    Mercenery,
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
            "Mercenery" => Ok(CharacterClass::Mercenery),
            "Ranger" => Ok(CharacterClass::Ranger),
            _ => Err(serde::de::Error::unknown_variant(
                &s,
                &[
                    "Monk",
                    "Sorceress",
                    "Witch",
                    "Warrior",
                    "Mercenery",
                    "Ranger",
                ],
            )),
        }
    }
}
