// in your src/lib.rs or src/main.rs:
mod stat;
use ahash::AHashMap;

pub use stat::Stat;

impl Stat {
    pub fn name(&self) -> &str {
        self.as_str()
    }
    /// Aggregates stats that are of PlusPercentage/Plus/MinusPercentage etc type into a AHashMap where the key is the stat name
    /// and the value is a tuple of (total value, count of nodes).
    pub fn aggregate_stats<'t>(
        stats: impl Iterator<Item = &'t Stat>,
    ) -> AHashMap<String, (f32, usize)> {
        let mut groups: AHashMap<String, (f32, usize)> = AHashMap::new();
        stats.for_each(|stat| {
            let (name, value) = (stat.as_str(), stat.value());

            groups
                .entry(name.to_owned())
                .and_modify(|(sum, count)| {
                    *sum += value;
                    *count += 1;
                })
                .or_insert((value, 1));
        });
        groups
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
    use crate::quick_tree;

    #[test]
    fn compute_all_evasion_rating_15_percents() {
        let mut tree = quick_tree();

        tree.remove_hidden();

        let stats = tree
            .nodes
            .values()
            .map(|pnode| pnode.as_passive_skill(&tree));

        let er_count: usize = stats
            .into_iter()
            .flat_map(|passive| passive.stats())
            .filter(|s| matches!(s, Stat::EvasionRating(_)))
            .count();

        println!("EvasionRating skills: {}", er_count);
    }

    #[test]
    fn compute_all_energy_shield_15_percents() {
        // _ = pretty_env_logger::init();
        let mut tree = quick_tree();
        tree.remove_hidden();
        let stats = tree.nodes.values().map(|p| p.as_passive_skill(&tree));
        let mut total = 0.0;
        let es_count = stats
            .flat_map(|ps| ps.stats())
            .filter(|s| matches!(s, Stat::MaximumEnergyShield(_)))
            .filter(|s| match s {
                Stat::MaximumEnergyShield(perc) => {
                    println!("{}", perc.0);

                    total += perc.0;
                    true
                }
                _ => false,
            })
            .count();

        println!("+% EnergyShield skills: {}, totalling {total}", es_count);
    }
}
