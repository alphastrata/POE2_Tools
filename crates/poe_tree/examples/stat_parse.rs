use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug)]
struct PlusPercentage(u16);

#[derive(Debug)]
enum Stat {
    EvasionRating(PlusPercentage),
    MaxEnergyShield(PlusPercentage),
    Unknown(String, u16),
}

#[derive(Debug)]
struct PassiveSkill {
    name: String,
    icon: String,
    stats: Vec<Stat>,
}

impl<'de> Deserialize<'de> for PassiveSkill {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // intermediate
        #[derive(Deserialize)]
        struct Inner {
            name: String,
            icon: String,
            stats: HashMap<String, u16>,
        }

        let inner = Inner::deserialize(deserializer)?;
        // convert each (key,value) into our enum
        let stats = inner
            .stats
            .into_iter()
            .map(|(k, v)| match k.as_str() {
                "evasion_rating_+%" => Stat::EvasionRating(PlusPercentage(v)),
                "maximum_energy_shield_+%" => Stat::MaxEnergyShield(PlusPercentage(v)),
                _ => Stat::Unknown(k, v),
            })
            .collect();

        Ok(PassiveSkill {
            name: inner.name,
            icon: inner.icon,
            stats,
        })
    }
}

fn main() {
    let json = r#"
    {
      "name": "Evasion and Energy Shield",
      "icon": "skillicons/passives/evasionandenergyshieldnode",
      "stats": {
        "evasion_rating_+%": 12,
        "maximum_energy_shield_+%": 12
      }
    }"#;
    let skill: PassiveSkill = serde_json::from_str(json).unwrap();
    println!("{:?}", skill);
}
