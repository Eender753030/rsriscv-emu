use crate::exception::Exception;

use super::Device;
use super::memory::{Memory, PAGE_SIZE};
use super::uart::Uart;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SystemBus {
    uart: Uart,
    ram: Memory,
}

pub const UART_BASE: u32 = 0x1000_0000;
pub const UART_END: u32 = 0x1000_00FF;
pub const DRAM_BASE_ADDR: u32 = 0x8000_0000;

impl SystemBus {
    pub fn mapping(&self, addr: u32) -> Result<(&dyn Device, u32), Exception> {
        match addr {
            UART_BASE..=UART_END => {
               let uart_addr = addr - UART_BASE;
               Ok((&self.uart, uart_addr))
            }
            DRAM_BASE_ADDR.. => {
                let ram_addr = addr - DRAM_BASE_ADDR;
                if ram_addr as usize >= self.ram.size {
                    Err(Exception::InstructionAccessFault)
                } else {
                    Ok((&self.ram, ram_addr))
                }
            },
            _ => Err(Exception::InstructionAccessFault),
        }
    }

    pub fn mapping_mut(&mut self, addr: u32) -> Result<(&mut dyn Device, u32), Exception> {
        match addr {
            UART_BASE..=UART_END => {
                let uart_addr = addr - UART_BASE;
                Ok((&mut self.uart, uart_addr))
            }
            DRAM_BASE_ADDR.. => {
                let ram_addr = addr - DRAM_BASE_ADDR;
                if ram_addr as usize >= self.ram.size {
                    Err(Exception::InstructionAccessFault)
                } else {
                    Ok((&mut self.ram, ram_addr))
                }
            },
            _ => Err(Exception::InstructionAccessFault),
        }
    }

    pub fn read_u32(&self, addr: u32) -> Result<u32, Exception> { 
        self.read_u32_bytes(addr, 4, false)
    }

    pub fn read_u32_bytes(&self, addr: u32, len: usize, is_signed: bool) -> Result<u32, Exception> {
        let (device, real_addr) = self.mapping(addr)?;

        let mut four_bytes = [0; 4];

        device.read_bytes(real_addr, len, &mut four_bytes[..len])?;

        if is_signed && (four_bytes[len - 1] & 0x80 != 0) {
            four_bytes[len..].fill(0xff);
        }

        Ok(u32::from_le_bytes(four_bytes))
    }

    #[allow(unused)]
    pub fn write_u32(&mut self, addr: u32, data: u32) -> Result<(), Exception> {
        self.write_u32_bytes(addr, data, 4)
    }

    pub fn write_u32_bytes(&mut self, addr: u32, data: u32, len: usize) -> Result<(), Exception> {
        let (device, real_addr) = self.mapping_mut(addr)?;
        device.write_bytes(real_addr, len, &data.to_le_bytes())?;
        Ok(())
    }

    pub fn ram_info(&self) -> (usize, u32, usize) {
        (self.ram.size, DRAM_BASE_ADDR, PAGE_SIZE)
    }

    pub fn reset_ram(&mut self) {
        self.ram.reset();
    }
}

impl Device for SystemBus {
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
