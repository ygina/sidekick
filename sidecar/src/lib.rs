use quack::*;

pub mod buffer;
mod sidecar;
pub mod sidecar_multi;
pub mod socket;

pub use buffer::ID_OFFSET;
pub use sidecar::Sidecar;
pub use sidecar_multi::SidecarMulti;
pub use socket::Socket;
