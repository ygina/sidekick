pub mod arithmetic {
    mod modint;
    mod evaluator;

    pub use modint::ModularInteger;
    pub use evaluator::MonicPolynomialEvaluator;
}

mod quack_internal;
pub use quack_internal::*;

pub type Identifier = u32;
pub type IdentifierLog = Vec<Identifier>;

pub trait Quack {
    fn new(threshold: usize) -> Self;
    fn insert(&mut self, value: Identifier);
    fn remove(&mut self, value: Identifier);
    fn threshold(&self) -> usize;
    fn count(&self) -> u16;
    fn decode(&self, log: &IdentifierLog) -> Vec<usize>;
}
