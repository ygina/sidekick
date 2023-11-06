pub mod arithmetic {
    mod evaluator;
    mod modint;
    #[cfg(feature = "montgomery")]
    mod montgomery;

    pub use evaluator::MonicPolynomialEvaluator;
    pub use modint::{ModularArithmetic, ModularInteger};
    #[cfg(feature = "montgomery")]
    pub use montgomery::MontgomeryInteger;
}

mod quack_internal;
#[cfg(feature = "montgomery")]
pub use quack_internal::MontgomeryQuack;
pub use quack_internal::PowerSumQuack;
#[cfg(feature = "power_table")]
pub use quack_internal::PowerTableQuack;
pub use quack_internal::StrawmanAQuack;
pub use quack_internal::StrawmanBQuack;
#[cfg(feature = "power_table")]
pub(crate) use quack_internal::{init_pow_table, POWER_TABLE};

pub trait Quack<T> {
    fn new(threshold: usize) -> Self;
    fn insert(&mut self, value: T);
    fn remove(&mut self, value: T);
    fn threshold(&self) -> usize;
    fn count(&self) -> u32;
    fn last_value(&self) -> T;

    fn decode_with_log(&self, log: &Vec<T>) -> Vec<T>;
    fn to_coeffs(&self) -> Vec<arithmetic::ModularInteger<T>>;
    fn to_coeffs_preallocated(&self, coeffs: &mut Vec<arithmetic::ModularInteger<T>>);
}
