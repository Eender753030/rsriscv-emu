use crate::error::RiscVError;

use super::memory::Memory;

pub trait Bus {
    fn read_byte(&mut self, addr: u32) -> Result<u8, RiscVError>;

    fn write_byte(&mut self, addr: u32, data: u8) -> Result<(), RiscVError>;

    fn read_bytes(&mut self, addr: u32, size: usize, des: &mut [u8]) -> Result<(), RiscVError>;

    fn write_bytes(&mut self, addr: u32, size: usize, src: &[u8]) -> Result<(), RiscVError>;
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SystemBus {
    pub ram: Memory,
}

impl SystemBus {
    pub fn new(ram_size: usize) -> Self {
        SystemBus {ram: Memory::new(ram_size)}
    }

    pub fn load_ins(&mut self, addr: u32, code: &[u8]) -> Result<(), RiscVError> {
        self.ram.write_bytes(addr, code.len(), code)
    }

    pub fn ram_read_u32(&mut self, addr: u32) -> Result<u32, RiscVError> {
        self.ram_read_u32_byte(addr, 4, false)
    }

    pub fn ram_read_u32_byte(&mut self, addr: u32, len: usize, is_signed: bool) -> Result<u32, RiscVError> {
        if !(1..=4).contains(&len) {
            return Err(RiscVError::ReadInvalidBytes);   
        }
    
        let mut four_bytes = [0; 4];

        self.ram.read_bytes(addr, len, &mut four_bytes[..len])?;

        if is_signed && (four_bytes.last().unwrap() & 0x80 != 0) {
            four_bytes[len..].fill(0xff);
        }

        Ok(u32::from_be_bytes(four_bytes))
    }

    pub fn ram_write_u32(&mut self, addr: u32, data: u32) -> Result<(), RiscVError> {
        self.ram.write_bytes(addr, 4, &data.to_le_bytes())
    }

    pub fn ram_fetch(&mut self, addr: u32) -> Result<u32, RiscVError> {
        if !addr.is_multiple_of(4) {
            Err(RiscVError::InstructionAddressMisaligned(addr))
        } else {
            self.ram_read_u32(addr)
        }
    } 
}