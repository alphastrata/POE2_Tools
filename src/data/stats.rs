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
