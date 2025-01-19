//$ crates/poe_tree/src/data/poe_tree/stats.rs
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Operand {
    Add,        // Represents "+"
    Multiply,   // Represents "x"
    Percentage, // Represents "+%"
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatType {
    Additive,
    Percentage,
    Grantable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stat {
    pub name: String,
    pub operand: Operand,
    pub value: f64,
}
impl Stat {
    pub fn apply(&self, other: &Stat) -> Option<f64> {
        if self.name == other.name {
            match self.operand {
                Operand::Add => Some(self.value + other.value),
                Operand::Multiply => Some(self.value * other.value),
                Operand::Percentage => Some(self.value + (self.value * other.value / 100.0)),
            }
        } else {
            None // Cannot apply operations on different stat types
        }
    }
}

// Custom deserializer for the stats field
pub(crate) fn deserialize_stats<'de, D>(deserializer: D) -> Result<Vec<Stat>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let map: HashMap<String, serde_json::Value> = HashMap::deserialize(deserializer)?;
    let mut stats = Vec::new();

    for (name, value) in map {
        // Parse the value and determine the operand
        let (operand, parsed_value) = match value {
            serde_json::Value::Number(n) => {
                let val = n.as_f64().unwrap_or(0.0);
                if name.contains('%') {
                    (Operand::Percentage, val)
                } else {
                    (Operand::Add, val)
                }
            }
            serde_json::Value::String(s) => {
                if let Ok(val) = s.parse::<f64>() {
                    if s.contains('x') {
                        (Operand::Multiply, val)
                    } else if s.contains('%') {
                        (Operand::Percentage, val)
                    } else {
                        (Operand::Add, val)
                    }
                } else {
                    continue; // Skip invalid values
                }
            }
            _ => continue, // Skip other types
        };

        stats.push(Stat {
            name,
            operand,
            value: parsed_value,
        });
    }

    Ok(stats)
}
