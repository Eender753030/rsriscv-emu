pub mod bus;
pub mod memory;
pub mod uart;

use crate::exception::Exception;

pub trait Device {

    fn read_byte(&self, addr: u32) -> Result<u8, Exception>;

    fn write_byte(&mut self, addr: u32, data: u8) -> Result<(), Exception>;

    fn read_bytes(&self, addr: u32, size: usize, des: &mut [u8]) -> Result<(), Exception>;

    fn write_bytes(&mut self, addr: u32, size: usize, src: &[u8]) -> Result<(), Exception>;
}