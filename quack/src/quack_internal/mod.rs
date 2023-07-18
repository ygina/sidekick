mod psum;
mod strawman_a;
#[cfg(feature = "power_table")]
mod ptable;

pub use psum::PowerSumQuack;
pub use strawman_a::StrawmanAQuack;
#[cfg(feature = "power_table")]
pub use ptable::PowerTableQuack;
#[cfg(feature = "power_table")]
pub(crate) use ptable::{init_pow_table, POWER_TABLE};
