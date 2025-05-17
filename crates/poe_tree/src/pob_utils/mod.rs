//! All human written code should go in here and branched off modules, don't touch the machine generated code in ./generated.rs

mod generated;

use std::{io::Write, path::Path};

pub use generated::POBCharacter;
use quick_xml::{de::from_str, se::to_string};

use crate::{character::Character, type_wrappings::NodeId};

impl Default for POBCharacter {
    fn default() -> Self {
        let xml = std::fs::read_to_string("../../data/empty-build.xml").unwrap();

        match from_str(&xml) {
            Ok(char) => char,
            Err(e) => {
                eprintln!("{e}");
                panic!("Unable to create a default character, this likely means the default character .xml is missing... not good not good! please file a bug report");
            }
        }
    }
}

impl POBCharacter {
    /// Exports this POBCharacter as XML and writes it to the specified path.
    pub fn export_for_pob<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let xml_string = to_string(&self)?;
        let mut file = std::fs::File::create(path)?;
        file.write_all(xml_string.as_bytes())?;
        Ok(())
    }

    /// NOTE: They (POB) store the nodes as a String :puke emoji, so this converts.
    pub fn activated_node_ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.tree
            .spec
            .nodes
            .split(',')
            .filter_map(|s| s.parse::<NodeId>().ok())
    }
}

impl From<Character> for POBCharacter {
    fn from(value: Character) -> Self {
        // create a default
        // override what we can

        todo!()
    }
}

impl From<POBCharacter> for Character {
    fn from(value: POBCharacter) -> Self {
        let mut base = Self::default();

        base.activated_node_ids = value.activated_node_ids().collect();
        base.level = value.build.level.parse().unwrap(); //TODO: Errors or TryFrom<T> -> X
        base.character_class = value.build.class_name;

        base.starting_node = *base
            .activated_node_ids
            .iter()
            .find(|node| crate::consts::LEVEL_ONE_NODES.contains(node))
            .expect("No activated node found in LEVEL_ONE_NODES");

        base
    }
}

//TODO: implement a poe_tree::Character -> POBCharacter conversion.

#[cfg(test)]
mod tests {
    use crate::{
        character::{Character, CharacterClass},
        consts::LEVEL_ONE_NODES,
        pob_utils::generated::POBCharacter,
        type_wrappings::NodeId,
    };
    use quick_xml::de::from_str;
    use std::fs;

    #[test]
    fn deserialize_monk_high_evasion() {
        let xml = std::fs::read_to_string("../../data/monk-high-evasion.xml").unwrap();
        let pob_char: POBCharacter = from_str(&xml).unwrap();
        assert_eq!(pob_char.build.class_name, CharacterClass::Monk);
        println!("{:#?}", pob_char);
    }

    #[test]
    fn can_import_pob_chars_active_nodes_manual() {
        let xml = fs::read_to_string("../../data/monk-high-evasion.xml")
            .expect("Unable to read monk-high-evasion.xml");
        let theirs: POBCharacter = from_str(&xml).expect("Failed to deserialize POBCharacter");

        let mut ours = Character::load_from_toml("../../data/character.toml")
            .expect("Failed to load character.toml");

        // override our ids with theirs.
        ours.activated_node_ids = theirs.activated_node_ids().collect();

        // Check that the expected node IDs are now present, and that they deserialised and parsed...
        "44683,33866,21336,42857,62984,49220,7576,10364,15975"
            .split(',')
            .filter_map(|s| s.parse::<NodeId>().ok())
            .for_each(|n| {
                assert!(
                    ours.activated_node_ids.contains(&n),
                    "Missing node id {}",
                    n
                )
            });

        // We determine the starting node by finding one that exists in LEVEL_ONE_NODES.
        let starting_candidate = ours
            .activated_node_ids
            .iter()
            .find(|node| LEVEL_ONE_NODES.contains(node))
            .expect("No activated node found in LEVEL_ONE_NODES");
        ours.starting_node = *starting_candidate;

        assert!(
            LEVEL_ONE_NODES.contains(&ours.starting_node),
            "Starting node {} is not in LEVEL_ONE_NODES",
            ours.starting_node
        );
    }

    #[test]
    fn can_import_pob_chars_from_impl() {
        // If _this_ test is broken see the one above it which more granularly demonstrates what goes into the From<T> impl
        // it is easier to debug the steps there.
        let xml = fs::read_to_string("../../data/monk-high-evasion.xml")
            .expect("Unable to read monk-high-evasion.xml");
        let theirs: POBCharacter = from_str(&xml).expect("Failed to deserialize POBCharacter");

        let ours: Character = theirs.into();

        // Check that the expected node IDs are now present, and that they deserialised and parsed...
        "44683,33866,21336,42857,62984,49220,7576,10364,15975"
            .split(',')
            .filter_map(|s| s.parse::<NodeId>().ok())
            .for_each(|n| {
                assert!(
                    ours.activated_node_ids.contains(&n),
                    "Missing node id {}",
                    n
                )
            });

        assert!(
            LEVEL_ONE_NODES.contains(&ours.starting_node),
            "Starting node {} is not in LEVEL_ONE_NODES",
            ours.starting_node
        );
    }

    #[test]
    fn can_parse_complex_with_jewel() {
        let xml = fs::read_to_string("../../data/complex-with-jewl.xml")
            .expect("Unable to read monk-high-evasion.xml");

        let _v: POBCharacter = from_str(&xml).unwrap();
    }
}
