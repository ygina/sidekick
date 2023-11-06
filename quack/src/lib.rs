#![feature(doc_cfg)]

//! The _quACK_ is a data structure for being able to refer to and efficiently
//! acknowledge a set of opaque packets seen by a network intermediary. The
//! recommended quACK implementation is the 32-bit power sum quACK with no
//! features.
//!
//! In the quACK problem, a data sender transmits a multiset (meaning the same
//! element can be transmitted more than once) of elements `S` (these
//! correspond to packets). At any given time, a receiver (such as a proxy
//! server) has received a subset `R \subseteq S` of the sent elements. We
//! would like the receiver to communicate a small amount of information to the
//! sender, who then efficiently decodes the missing elements---the set
//! difference `S \ R`---knowing `S`. This small amount of information is called
//! the _quACK_ and the problem is: what is in a quACK and how do we decode it?

#[macro_use]
mod macros;

/// Efficient modular arithmetic and polynomial evaluation.
pub mod arithmetic {
    mod evaluator;
    mod modint;

    pub use evaluator::MonicPolynomialEvaluator;
    pub use modint::{ModularArithmetic, ModularInteger};

    cfg_montgomery! {
        mod montgomery;
        pub use montgomery::MontgomeryInteger;
    }
}

mod psum;
mod strawman_a;
mod strawman_b;

pub use psum::PowerSumQuack;
pub use strawman_a::StrawmanAQuack;
pub use strawman_b::StrawmanBQuack;

cfg_montgomery! {
    mod montgomery;
    pub use montgomery::MontgomeryQuack;
}

cfg_power_table! {
    mod ptable;
    pub use ptable::PowerTableQuack;
    pub(crate) use ptable::POWER_TABLE;
}
