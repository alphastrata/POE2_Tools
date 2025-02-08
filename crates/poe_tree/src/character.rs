use crate::{pob_utils, stats::Stat, type_wrappings::NodeId, PassiveTree};
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
    pub level: u8,
    pub quest_passive_skills: u8,
    pub starting_node: NodeId,
}

impl Character {
    fn load_pob_empty(file: &str) -> Self {
        let xml = std::fs::read_to_string(file).unwrap();
        let pob_char: pob_utils::POBCharacter = quick_xml::de::from_str(&xml).unwrap();
        pob_char.into()
    }

    pub fn default_sorceress() -> Self {
        Self::load_pob_empty("../../data/empty-sorceress.xml")
    }
    pub fn default_witch() -> Self {
        Self::default_sorceress()
    }
    pub fn default_monk() -> Self {
        Self::load_pob_empty("../../data/empty-monk.xml")
    }
    pub fn default_ranger() -> Self {
        Self::load_pob_empty("../../data/empty-ranger.xml")
    }
    pub fn default_warrior() -> Self {
        Self::load_pob_empty("../../data/empty-warrior.xml")
    }

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

    pub fn compute_stats(&self, tree: &PassiveTree) -> CharacterStats {
        let mut start = CharacterStats::default_monk();

        start.level = self.level;
        start.name = self.name.clone();

        start
    }

    pub fn all_stats<'t>(&'t self, tree: &'t PassiveTree) -> impl Iterator<Item = &'t Stat> + '_ {
        self.activated_node_ids
            .iter()
            .map(|nid| tree.node(*nid))
            .map(|pnode| tree.passive_for_node(pnode))
            .flat_map(|pnode| pnode.stats())
    }

    pub fn all_stats_with_ids<'t>(
        &'t self,
        tree: &'t PassiveTree,
    ) -> impl Iterator<Item = (NodeId, &'t [Stat])> + '_ {
        self.activated_node_ids
            .iter()
            .map(|nid| tree.node(*nid))
            .map(|pnode| (pnode.node_id, tree.passive_for_node(pnode)))
            .map(|(nid, skill)| (nid, skill.stats()))
    }

    pub fn calculate_energy_shield(&self, tree: &PassiveTree) -> f32 {
        todo!()
    }

    pub fn calculate_life(&self, tree: &PassiveTree) -> f32 {
        todo!()
    }

    pub fn calcluate_attack_speed(&self, tree: &PassiveTree) -> f32 {
        todo!()
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

#[cfg(test)]
mod tests {
    use crate::quick_tree;
    use crate::stats::arithmetic::*;
    use crate::stats::*;

    use super::Character;
    const TEST_DATA_MONK: &str = "../../data/character.toml";

    #[test]
    fn sum_evasion_rating_plus_percentage() {
        let tree = quick_tree();
        let lvl_13_mostly_evasion_nodes = vec![
            15975, 62984, 49220, 10364, 5702, 20024, 44223, 48198, 21336, 42857, 13411, 56045,
            24647,
        ];
        let mut char = Character::load_from_toml(TEST_DATA_MONK).unwrap();
        char.activated_node_ids = lvl_13_mostly_evasion_nodes.into_iter().collect();

        let stats = char.all_stats(&tree);
        let mut total_evasion = PlusPercentage(0.0);

        for stat in stats {
            if let Stat::EvasionRating(PlusPercentage(val)) = stat {
                total_evasion += PlusPercentage(*val);
            }
        }

        // e.g. just check it's > 0
        assert!(total_evasion.0 > 0.0);
        println!("Total EvasionRating(PlusPercentage): {}", total_evasion.0);
    }

    #[test]
    fn sums_of_various_stats() {
        let tree = quick_tree();
        let lvl_13_mostly_evasion_nodes = vec![
            15975, 62984, 49220, 10364, 5702, 20024, 44223, 48198, 21336, 42857, 13411, 56045,
            24647,
        ];
        let mut char = Character::load_from_toml(TEST_DATA_MONK).unwrap();
        char.activated_node_ids = lvl_13_mostly_evasion_nodes.into_iter().collect();

        let stats = char.all_stats(&tree);

        let (mut sum_evasion, mut count_evasion) = (PlusPercentage(0.0), 0);
        let (mut sum_es, mut count_es) = (PlusPercentage(0.0), 0);
        let (mut sum_skill_speed, mut count_skill_speed) = (PlusPercentage(0.0), 0);
        let (mut sum_attack_cast_speed, mut count_attack_cast_speed) = (PlusPercentage(0.0), 0);

        for stat in stats {
            match stat {
                Stat::EvasionRating(PlusPercentage(val)) => {
                    sum_evasion += PlusPercentage(*val);
                    count_evasion += 1;
                }
                Stat::MaximumEnergyShield(PlusPercentage(val)) => {
                    sum_es += PlusPercentage(*val);
                    count_es += 1;
                }
                Stat::SkillSpeed(PlusPercentage(val)) => {
                    sum_skill_speed += PlusPercentage(*val);
                    count_skill_speed += 1;
                }
                Stat::AttackAndCastSpeed(PlusPercentage(val)) => {
                    sum_attack_cast_speed += PlusPercentage(*val);
                    count_attack_cast_speed += 1;
                }
                _ => {}
            }
        }

        println!(
            "Sum of EvasionRating(PlusPercentage): {} from {:?} nodes",
            sum_evasion.0, count_evasion
        );
        println!(
            "Sum of MaximumEnergyShield(PlusPercentage): {} from {:?} nodes",
            sum_es.0, count_es
        );
        println!(
            "Sum of SkillSpeed(PlusPercentage): {} from {:?} nodes",
            sum_skill_speed.0, count_skill_speed
        );
        println!(
            "Sum of AttackAndCastSpeed(PlusPercentage): {} from {:?} nodes",
            sum_attack_cast_speed.0, count_attack_cast_speed
        );
    }

    #[test]
    fn sums_of_various_stats_and_the_ids_that_make_them_nicely_formatted() {
        let tree = quick_tree();
        let lvl_13_mostly_evasion_nodes = vec![
            15975, 62984, 49220, 10364, 5702, 20024, 44223, 48198, 21336, 42857, 13411, 56045,
            24647,
        ];
        let mut char = Character::load_from_toml(TEST_DATA_MONK).unwrap();
        char.activated_node_ids = lvl_13_mostly_evasion_nodes.into_iter().collect();

        let stats_with_ids = char.all_stats_with_ids(&tree);

        let (mut sum_evasion, mut count_evasion) = (PlusPercentage(0.0), 0);
        let (mut sum_es, mut count_es) = (PlusPercentage(0.0), 0);
        let (mut sum_skill_speed, mut count_skill_speed) = (PlusPercentage(0.0), 0);
        let (mut sum_attack_cast_speed, mut count_attack_cast_speed) = (PlusPercentage(0.0), 0);

        stats_with_ids.for_each(|(node_id, stat_slice)| {
            let mut lines = vec![];
            for stat in stat_slice {
                match stat {
                    Stat::EvasionRating(PlusPercentage(val)) => {
                        sum_evasion += PlusPercentage(*val);
                        count_evasion += 1;
                        lines.push(format!("EvasionRating +{}%", val));
                    }
                    Stat::MaximumEnergyShield(PlusPercentage(val)) => {
                        sum_es += PlusPercentage(*val);
                        count_es += 1;
                        lines.push(format!("MaxEnergyShield +{}%", val));
                    }
                    Stat::SkillSpeed(PlusPercentage(val)) => {
                        sum_skill_speed += PlusPercentage(*val);
                        count_skill_speed += 1;
                        lines.push(format!("SkillSpeed +{}%", val));
                    }
                    Stat::AttackAndCastSpeed(PlusPercentage(val)) => {
                        sum_attack_cast_speed += PlusPercentage(*val);
                        count_attack_cast_speed += 1;
                        lines.push(format!("AttackAndCastSpeed +{}%", val));
                    }
                    _ => {}
                }
            }
            if !lines.is_empty() {
                println!("Node {} contributed:", node_id);
                for l in lines {
                    println!("  {}", l);
                }
            }
        });

        println!("{}\nSUMMARY:", "*".repeat(80));
        println!(
            "  Sum of EvasionRating(PlusPercentage): +{}% from {} nodes",
            sum_evasion.0, count_evasion
        );
        println!(
            "  Sum of MaximumEnergyShield(PlusPercentage): +{}% from {} nodes",
            sum_es.0, count_es
        );
        println!(
            "  Sum of SkillSpeed(PlusPercentage): +{}% from {} nodes",
            sum_skill_speed.0, count_skill_speed
        );
        println!(
            "  Sum of AttackAndCastSpeed(PlusPercentage): +{}% from {} nodes",
            sum_attack_cast_speed.0, count_attack_cast_speed
        );
    }
}
