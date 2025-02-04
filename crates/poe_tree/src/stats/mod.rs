// in your src/lib.rs or src/main.rs:
mod stat;
pub use stat::Stat;

impl Stat {
    pub fn name(&self) -> String {
        todo!()
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
