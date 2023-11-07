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

pub(crate) mod precompute;
pub use precompute::global_config_set_max_power_sum_threshold;

/// Efficient modular arithmetic and polynomial evaluation.
pub mod arithmetic {
    mod evaluator;
    mod modint;

    pub use evaluator::*;
    pub use modint::{ModularArithmetic, ModularInteger};

    cfg_montgomery! {
        mod montgomery;
        pub use montgomery::MontgomeryInteger;
    }
}

mod power_sum;
pub use power_sum::{PowerSumQuack, PowerSumQuackU32};

cfg_strawmen! {
    mod strawmen;
    pub use strawmen::StrawmanAQuack;
    pub use strawmen::StrawmanBQuack;
}

cfg_montgomery! {
    mod montgomery;
    pub use power_sum::PowerSumQuackU64;
    pub use montgomery::MontgomeryQuack;
}

cfg_power_table! {
    mod power_table;
    pub use power_sum::PowerSumQuackU16;
    pub use power_table::PowerTableQuack;
}
