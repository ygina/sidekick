mod psum;
#[cfg(feature = "power_table")]
mod ptable;

pub use psum::PowerSumQuack;
#[cfg(feature = "power_table")]
pub use ptable::PowerTableQuack;
#[cfg(feature = "power_table")]
pub(crate) use ptable::{init_pow_table, POWER_TABLE};
