// in your src/lib.rs or src/main.rs:
mod stat;
pub use stat::Stat;

impl Stat {
    pub fn name(&self) -> &str {
        self.as_str()
    }
}
// In your src/lib.rs:
pub mod arithmetic {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Plus(pub f32);
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Minus(pub f32);
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Multiply(pub f32);
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Divide(pub f32);
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct PlusPercentage(pub f32);
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct MinusPercentage(pub f32);
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Other(pub f32);

    macro_rules! impl_basic_maths_for {
        ($ty:ident) => {
            impl std::ops::Add for $ty {
                type Output = Self;
                fn add(self, rhs: Self) -> Self {
                    Self(self.0 + rhs.0)
                }
            }
            impl std::ops::AddAssign for $ty {
                fn add_assign(&mut self, rhs: Self) {
                    self.0 += rhs.0;
                }
            }
        };
    }
    impl_basic_maths_for!(Plus);
    impl_basic_maths_for!(Minus);
    impl_basic_maths_for!(Multiply);
    impl_basic_maths_for!(Divide);
    impl_basic_maths_for!(PlusPercentage);
    impl_basic_maths_for!(MinusPercentage);
    impl_basic_maths_for!(Other);
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{quick_tree, stats::arithmetic::*};

    #[test]
    fn compute_all_evasion_rating_15_percents() {
        _ = pretty_env_logger::init();

        let mut tree = quick_tree();

        tree.remove_hidden();

        let stats = tree
            .nodes
            .iter()
            .map(|(_nid, pnode)| pnode.as_passive_skill(&tree));

        let er_count: usize = stats
            .into_iter()
            .flat_map(|passive| passive.stats())
            .filter(|s| matches!(s, Stat::EvasionRating(_)))
            .count();

        println!("EvasionRating skills: {}", er_count);
    }

    #[test]
    fn compute_all_energy_shield_15_percents() {
        _ = pretty_env_logger::init();
        let mut tree = quick_tree();
        tree.remove_hidden();
        let stats = tree.nodes.iter().map(|(_, p)| p.as_passive_skill(&tree));
        let es_count = stats
            .flat_map(|ps| ps.stats())
            .filter(|s| matches!(s, Stat::MaximumEnergyShield(_)))
            // At this stage it could be Plus or PlusPercentage
            .filter(|s| match s {
                /*  "maximum_energy_shield_+%" => Stat::MaximumEnergyShield(PlusPercentage(v as f32)), */
                Stat::MaximumEnergyShield(plus) => true,
                // Stat::MaximumEnergyShieldFromBodyArmour(plus) => true,
                _ => false,
            })
            .count();

        println!("+% EnergyShield skills: {}", es_count);
    }

    #[test]
    fn compute_both_and_sum_plus_percentage() {
        _ = pretty_env_logger::init();
        let mut tree = quick_tree();
        tree.remove_hidden();
        let stats = tree.nodes.iter().map(|(_, p)| p.as_passive_skill(&tree));
        let mut total = 0f32;
        // for s in stats.flat_map(|ps| ps.stats()) {
        //     match s {
        //         Stat::Evasion(PlusPercentage(val)) | Stat::EnergyShield(Plus(val)) => total += val,
        //         _ => {}
        //     }
        // }
        // println!(
        //     "Sum of EvasionRating and MaxEnergyShield (PlusPercentage): {}",
        //     total
        // );
    }
}
