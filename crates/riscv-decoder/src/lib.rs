pub mod decoder;
pub mod instruction;
pub mod prelude;

mod bits_op;
mod error;
mod opcode;

pub use error::DecodeError;
