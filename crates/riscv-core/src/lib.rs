mod core;
mod device;
mod engine;
mod error;
mod exception;

pub mod debug;
pub mod prelude;
pub mod constance {
    pub use crate::device::memory::PAGE_SIZE;
    pub use crate::device::bus::DRAM_BASE_ADDR;
}

pub use core::RiscV;
pub use error::RiscVError;
pub use exception::Exception;
pub use exception::Result;