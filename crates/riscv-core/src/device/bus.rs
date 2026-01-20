use crate::exception::Exception;

use super::memory::Memory;

pub trait Bus {
    fn read_byte(&self, addr: u32) -> Result<u8, Exception>;

    fn write_byte(&mut self, addr: u32, data: u8) -> Result<(), Exception>;

    fn read_bytes(&self, addr: u32, size: usize, des: &mut [u8]) -> Result<(), Exception>;

    fn write_bytes(&mut self, addr: u32, size: usize, src: &[u8]) -> Result<(), Exception>;
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SystemBus {
    ram: Memory,
}

const DRAM_BASE_ADDR: u32 = 0x8000_0000;

impl SystemBus {
    pub fn mapping(&self, addr: u32) -> Result<(Box<&dyn Bus>, u32), Exception> {
        match addr {
            DRAM_BASE_ADDR.. => {
                let ram_addr = addr - DRAM_BASE_ADDR;
                if ram_addr as usize >= self.ram.size {
                    Err(Exception::InstructionAccessFault)
                } else {
                    Ok((Box::new(&self.ram), ram_addr))
                }
            },
            _ => Err(Exception::InstructionAccessFault),
        }
    }

    pub fn mapping_mut(&mut self, addr: u32) -> Result<(Box<&mut dyn Bus>, u32), Exception> {
        match addr {
            DRAM_BASE_ADDR.. => {
                let ram_addr = addr - DRAM_BASE_ADDR;
                if ram_addr as usize >= self.ram.size {
                    Err(Exception::InstructionAccessFault)
                } else {
                    Ok((Box::new(&mut self.ram), ram_addr))
                }
            },
            _ => Err(Exception::InstructionAccessFault),
        }
    }

    pub fn read_u32(&mut self, addr: u32) -> Result<u32, Exception> { 
        self.read_u32_bytes(addr, 4, false)
    }

    pub fn read_u32_bytes(&mut self, addr: u32, len: usize, is_signed: bool) -> Result<u32, Exception> {
        let (device, real_addr) = self.mapping_mut(addr)?;

        let mut four_bytes = [0; 4];

        device.read_bytes(real_addr, len, &mut four_bytes[..len])?;

        if is_signed && (four_bytes.last().unwrap() & 0x80 != 0) {
            four_bytes[len..].fill(0xff);
        }

        Ok(u32::from_le_bytes(four_bytes))
    }

    pub fn write_u32(&mut self, addr: u32, data: u32) -> Result<(), Exception> {
        self.write_u32_bytes(addr, data, 4)
    }

    pub fn write_u32_bytes(&mut self, addr: u32, data: u32, len: usize) -> Result<(), Exception> {
        let (device, real_addr) = self.mapping_mut(addr)?;
        device.write_bytes(real_addr, len, &data.to_le_bytes())?;
        Ok(())
    }

    pub fn reset_ram(&mut self) {
        self.ram.reset();
    }
}

impl Bus for SystemBus {
    fn read_byte(&self, addr: u32) -> Result<u8, Exception> {
        let (device, real_addr) = self.mapping(addr)?;
        device.read_byte(real_addr)
    }

    fn write_byte(&mut self, addr: u32, data: u8) -> Result<(), Exception> {
        let (device, real_addr) = self.mapping_mut(addr)?;
        device.write_byte(real_addr, data)
    }

    fn read_bytes(&self, addr: u32, size: usize, des: &mut [u8]) -> Result<(), Exception> {
        let (device, real_addr) = self.mapping(addr)?;
        device.read_bytes(real_addr, size, des)
    }

    fn write_bytes(&mut self, addr: u32, size: usize, src: &[u8]) -> Result<(), Exception> {
        let (device, real_addr) = self.mapping_mut(addr)?;
        device.write_bytes(real_addr, size, src)
    }
}
