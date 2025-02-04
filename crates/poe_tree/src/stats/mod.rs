pub mod stats_enum;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Operand {
    Add,      // Represents "+"
    Multiply, // Represents "x"
    Percentage,
    Unhandled, // Represents "+%"
}

#[deprecated]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stat {
    pub name: String,
    pub operand: Operand,
    pub value: f32,
}

impl Stat {
    // StatA + StatB, assuming they have the same name...
    //NOTE: Does not handle the instance of StatA + StatB where one of them has ADDITIONAL values, therefore a different passive_skill name.
    pub fn plus(&self, other: &Stat) -> Option<f32> {
        if self.name == other.name {
            match self.operand {
                Operand::Add => Some(self.value + other.value),
                Operand::Multiply => Some(self.value * other.value),
                Operand::Percentage => Some(self.value + (self.value * other.value / 100.0)),
                _ => None,
            }
        } else {
            None // Cannot apply operations on different stat types
        }
    }
}

// Custom deserializer for the stats field
pub fn deserialize_stats<'de, D>(deserializer: D) -> Result<Vec<Stat>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let map: HashMap<String, serde_json::Value> = HashMap::deserialize(deserializer)?;
    let mut stats: Vec<Stat> = Vec::new();

    for (name, value) in map {
        // Parse the value and determine the operand
        let (operand, parsed_value) = match &value {
            serde_json::Value::Number(n) => {
                // Value doesn't support deserialising to f32
                let val = n.as_f64().unwrap_or(0.0) as f32;
                if name.contains('%') {
                    (Operand::Percentage, val)
                } else if name.contains("+") {
                    (Operand::Add, val)
                } else {
                    log::warn!("'n' Unhandled Stat type {:#?}", &value);
                    (Operand::Unhandled, val)
                }
            }
            serde_json::Value::String(s) => {
                if let Ok(val) = s.parse::<f32>() {
                    if s.contains('x') {
                        (Operand::Multiply, val)
                    } else if s.contains('%') {
                        (Operand::Percentage, val)
                    } else {
                        (Operand::Add, val)
                    }
                } else {
                    log::warn!("'s' Unhandled Stat type {:#?}", &value);
                    continue;
                }
            }
            _ => {
                log::warn!("'x' Unhandled Stat type {:#?}", &value);
                continue;
            }
        };

        stats.push(Stat {
            name,
            operand,
            value: parsed_value,
        });
    }

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::stats::stats_enum::{Plus, PlusPercentage, StatType};

    #[test]
    fn test_evasion_rating_while_surrounded() {
        let data = json!({ "name": "evasion_rating_+%", "value": 15.0 });
        let parsed: StatType = serde_json::from_value(data).unwrap();
        assert_eq!(parsed, StatType::evasion_rating(PlusPercentage(15.0)));
    }

    #[test]
    fn test_melee_range_minus() {
        let data = json!({ "name": "melee_range", "value": 3.0 });
        let parsed: StatType = serde_json::from_value(data).unwrap();
        assert_eq!(parsed, StatType::melee_range(Plus(3.0)));
    }

    #[test]
    fn test_critical_strike_multiplier_with_dagger_plus() {
        let data = json!({ "name": "critical_strike_multiplier_with_dagger", "value": 12.0 });
        let parsed: StatType = serde_json::from_value(data).unwrap();
        assert_eq!(
            parsed,
            StatType::critical_strike_multiplier_with_dagger(Plus(12.0))
        );
    }

    #[test]
    fn test_base_spell_critical_strike_multiplier_plus() {
        let data = json!({ "name": "base_spell_critical_strike_multiplier", "value": 20.0 });
        let parsed: StatType = serde_json::from_value(data).unwrap();
        assert_eq!(
            parsed,
            StatType::base_spell_critical_strike_multiplier(Plus(20.0))
        );
    }

    #[test]
    fn test_one_handed_melee_critical_strike_multiplier_plus() {
        let data = json!({ "name": "one_handed_melee_critical_strike_multiplier", "value": 5.0 });
        let parsed: StatType = serde_json::from_value(data).unwrap();
        assert_eq!(
            parsed,
            StatType::one_handed_melee_critical_strike_multiplier(Plus(5.0))
        );
    }

    #[test]
    fn test_some_unrecognized_stat_falls_back_to_error() {
        let data = json!({ "name": "completely_unknown_stat_name", "value": 99.0 });
        let parsed = serde_json::from_value::<StatType>(data);
        assert!(
            parsed.is_err(),
            "Expected unknown stat to fail deserialization"
        );
    }

    #[test]
    fn test_sum_operation_plus() {
        // If they're all the same variant, sum should unify them
        let stats = vec![
            StatType::map_pinnacle_boss_difficulty(Plus(8.0)),
            StatType::map_pinnacle_boss_difficulty(Plus(2.0)),
        ];
        let result = StatType::sum(&stats).unwrap();
        assert_eq!(result, StatType::map_pinnacle_boss_difficulty(Plus(10.0)));
    }

    #[test]
    fn test_add_operation_plus() {
        let a = StatType::map_pinnacle_boss_difficulty(Plus(8.0));
        let b = StatType::map_pinnacle_boss_difficulty(Plus(2.0));
        let result = a + b;
        assert_eq!(
            result,
            Some(StatType::map_pinnacle_boss_difficulty(Plus(10.0)))
        );
    }

    #[test]
    fn test_add_assign_operation_plus() {
        let mut a = StatType::map_pinnacle_boss_difficulty(Plus(8.0));
        let b = StatType::map_pinnacle_boss_difficulty(Plus(2.0));
        a += b;
        assert_eq!(a, StatType::map_pinnacle_boss_difficulty(Plus(10.0)));
    }
}
