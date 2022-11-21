pub mod arithmetic {
    mod modint;
    mod evaluator;

    pub use modint::ModularInteger;
    pub use evaluator::MonicPolynomialEvaluator;

}

mod quack;
mod decoded_quack;

pub use crate::quack::Quack;
pub use decoded_quack::DecodedQuack;