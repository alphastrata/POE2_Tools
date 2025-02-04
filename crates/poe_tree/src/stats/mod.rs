pub mod stats_enum;

pub type Stat = stats_enum::StatType;

impl Stat {
    pub fn name(&self) -> String {
        "".into()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::stats::stats_enum::{Plus, PlusPercentage, StatType};

    #[test]
    fn evasion_rating_while_surrounded() {
        let data = json!({ "name": "evasion_rating_+%", "value": 15.0 });
        let parsed: StatType = serde_json::from_value(data).unwrap();
        assert_eq!(parsed, StatType::evasion_rating(PlusPercentage(15.0)));
    }

    #[test]
    fn melee_range_minus() {
        let data = json!({ "name": "melee_range", "value": 3.0 });
        let parsed: StatType = serde_json::from_value(data).unwrap();
        assert_eq!(parsed, StatType::melee_range(Plus(3.0)));
    }

    #[test]
    fn critical_strike_multiplier_with_dagger_plus() {
        let data = json!({ "name": "critical_strike_multiplier_with_dagger", "value": 12.0 });
        let parsed: StatType = serde_json::from_value(data).unwrap();
        assert_eq!(
            parsed,
            StatType::critical_strike_multiplier_with_dagger(Plus(12.0))
        );
    }

    #[test]
    fn base_spell_critical_strike_multiplier_plus() {
        let data = json!({ "name": "base_spell_critical_strike_multiplier", "value": 20.0 });
        let parsed: StatType = serde_json::from_value(data).unwrap();
        assert_eq!(
            parsed,
            StatType::base_spell_critical_strike_multiplier(Plus(20.0))
        );
    }

    #[test]
    fn one_handed_melee_critical_strike_multiplier_plus() {
        let data = json!({ "name": "one_handed_melee_critical_strike_multiplier", "value": 5.0 });
        let parsed: StatType = serde_json::from_value(data).unwrap();
        assert_eq!(
            parsed,
            StatType::one_handed_melee_critical_strike_multiplier(Plus(5.0))
        );
    }

    #[test]
    fn some_unrecognized_stat_falls_back_to_error() {
        let data = json!({ "name": "completely_unknown_stat_name", "value": 99.0 });
        let parsed = serde_json::from_value::<StatType>(data);
        assert!(
            parsed.is_err(),
            "Expected unknown stat to fail deserialization"
        );
    }

    #[test]
    fn sum_operation_plus() {
        // If they're all the same variant, sum should unify them
        let stats = vec![
            StatType::map_pinnacle_boss_difficulty(Plus(8.0)),
            StatType::map_pinnacle_boss_difficulty(Plus(2.0)),
        ];
        let result = StatType::sum(&stats).unwrap();
        assert_eq!(result, StatType::map_pinnacle_boss_difficulty(Plus(10.0)));
    }

    #[test]
    fn add_operation_plus() {
        let a = StatType::map_pinnacle_boss_difficulty(Plus(8.0));
        let b = StatType::map_pinnacle_boss_difficulty(Plus(2.0));
        let result = a + b;
        assert_eq!(
            result,
            Some(StatType::map_pinnacle_boss_difficulty(Plus(10.0)))
        );
    }

    #[test]
    fn add_assign_operation_plus() {
        let mut a = StatType::map_pinnacle_boss_difficulty(Plus(8.0));
        let b = StatType::map_pinnacle_boss_difficulty(Plus(2.0));
        a += b;
        assert_eq!(a, StatType::map_pinnacle_boss_difficulty(Plus(10.0)));
    }
}
