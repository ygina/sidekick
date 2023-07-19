mod psum;
mod strawman_a;
mod strawman_b;
#[cfg(feature = "power_table")]
mod ptable;

pub use psum::PowerSumQuack;
pub use strawman_a::StrawmanAQuack;
pub use strawman_b::StrawmanBQuack;
#[cfg(feature = "power_table")]
pub use ptable::PowerTableQuack;
#[cfg(feature = "power_table")]
pub(crate) use ptable::{init_pow_table, POWER_TABLE};
