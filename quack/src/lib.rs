pub mod arithmetic {
    mod modint;
    mod evaluator;

    pub use modint::ModularInteger;
    pub use evaluator::MonicPolynomialEvaluator;

}

mod psum;
mod montgomery;
mod decoded_quack;

pub use crate::psum::PowerSumQuack;
pub use crate::montgomery::MontgomeryQuack;
pub use decoded_quack::{DecodedQuack, IdentifierLog};

pub type Identifier = u32;

pub trait Quack {
    fn new(threshold: usize) -> Self;
    fn insert(&mut self, value: Identifier);
    fn remove(&mut self, value: Identifier);
    fn threshold(&self) -> usize;
    fn count(&self) -> u16;
}
