#[cfg(feature = "montgomery")]
mod montgomery;
mod psum;
#[cfg(feature = "power_table")]
mod ptable;
mod strawman_a;
mod strawman_b;

#[cfg(feature = "montgomery")]
pub use montgomery::MontgomeryQuack;
pub use psum::PowerSumQuack;
#[cfg(feature = "power_table")]
pub use ptable::PowerTableQuack;
#[cfg(feature = "power_table")]
pub(crate) use ptable::{init_pow_table, POWER_TABLE};
pub use strawman_a::StrawmanAQuack;
pub use strawman_b::StrawmanBQuack;
