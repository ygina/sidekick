#![feature(doc_cfg)]
#![feature(const_trait_impl)]

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
    // mod evaluator;
    mod modint;

    // pub use evaluator::MonicPolynomialEvaluator;
    pub use modint::{ModularArithmetic, ModularInteger};

    // cfg_montgomery! {
    //     mod montgomery;
    //     pub use montgomery::MontgomeryInteger;
    // }
}

// mod psum;
// /// 32-bit power sum quACK.
// pub use psum::PowerSumQuack;
// /// 32-bit power sum quACK.
// pub struct PowerSumQuackU32 {}
// /// 64-bit power sum quACK.
// pub struct PowerSumQuackU64 {}
// /// 16-bit power sum quACK.
// pub struct PowerSumQuackU16 {}

// cfg_strawmen! {
//     mod strawman_a;
//     mod strawman_b;
//     /// Strawman quACK implementation that echoes every packet identifier.
//     pub use strawman_a::StrawmanAQuack;
//     /// Strawman quACK implementation that echoes a sliding window of packet identifiers.
//     pub use strawman_b::StrawmanBQuack;
// }

// cfg_montgomery! {
//     mod montgomery;
//     /// 64-bit power sum quACK using the Montgomery multiplication optimization.
//     pub use montgomery::MontgomeryQuack;
// }

// cfg_power_table! {
//     mod ptable;
//     /// 16-bit power sum quACK using the precomputation optimization.
//     pub use ptable::PowerTableQuack;
//     pub(crate) use ptable::POWER_TABLE;
// }
