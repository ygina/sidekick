pub mod arithmetic {
    mod modint;
    mod evaluator;

    pub use modint::{ModularInteger, ModularArithmetic};
    pub use evaluator::MonicPolynomialEvaluator;
}

mod quack_internal;
pub use quack_internal::{PowerSumQuack, PowerTableQuack};
#[cfg(feature = "power_table")]
pub(crate) use quack_internal::{init_pow_table, POWER_TABLE};

pub trait Quack<T> {
    fn new(threshold: usize) -> Self;
    fn insert(&mut self, value: T);
    fn remove(&mut self, value: T);
    fn threshold(&self) -> usize;
    fn count(&self) -> u16;

    fn decode_with_log(&self, log: &Vec<T>) -> Vec<T>;
    fn to_coeffs(&self) -> Vec<arithmetic::ModularInteger<T>>;
    fn to_coeffs_preallocated(&self, coeffs: &mut Vec<arithmetic::ModularInteger<T>>);
}
