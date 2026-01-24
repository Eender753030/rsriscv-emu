pub mod bus;
pub mod memory;
pub mod uart;

use crate::{core::{Access, Physical}, exception::Exception};

pub trait Device {
    fn read_byte(&self, access: Access<Physical>) -> Result<u8, Exception>;

    fn write_byte(&mut self, access: Access<Physical>, data: u8) -> Result<(), Exception>;

    fn read_bytes(&self, access: Access<Physical>, size: usize, des: &mut [u8]) -> Result<(), Exception>;

    fn write_bytes(&mut self, access: Access<Physical>, size: usize, src: &[u8]) -> Result<(), Exception>;
}