use crate::core::{Access, Physical};
use crate::exception::Exception;
use super::Device;
use super::memory::{Memory, PAGE_SIZE};
use super::uart::Uart;

use MappedDevice::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MappedDevice {
    Uart,
    Ram,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SystemBus {
    uart: Uart,
    ram: Memory,
}

pub const UART_BASE: u32 = 0x1000_0000;
pub const UART_END: u32 = 0x1000_00FF;
pub const DRAM_BASE_ADDR: u32 = 0x8000_0000;

impl SystemBus {
    fn mapping(&self, access: &mut Access<Physical>) -> Result<MappedDevice, Exception> {
        let addr = access.addr;
        Ok(match addr {
            UART_BASE..=UART_END => {
               access.addr = addr - UART_BASE;
               Uart
            }
            DRAM_BASE_ADDR.. => {
                let ram_addr = addr - DRAM_BASE_ADDR;
                if ram_addr as usize >= self.ram.size {
                    return Err(access.into_access_exception())
                } else {
                    access.addr = ram_addr;
                    Ram
                }
            },
            _ => return Err(access.into_access_exception()),
        })
    }

    pub fn read_u32(&self, access: Access<Physical>) -> Result<u32, Exception> { 
        self.read_u32_bytes(access, 4, false)
    }

    pub fn read_u32_bytes(&self, mut access: Access<Physical>, len: usize, is_signed: bool) -> Result<u32, Exception> {
        let mut four_bytes = [0; 4];

        match self.mapping(&mut access)? {
            Uart => self.uart.read_bytes(access, len, &mut four_bytes[..len])?,
            Ram  => self.ram.read_bytes(access, len, &mut four_bytes[..len])?,
        }
        
        if is_signed && (four_bytes[len - 1] & 0x80 != 0) {
            four_bytes[len..].fill(0xff);
        }

        Ok(u32::from_le_bytes(four_bytes))
    }

    pub fn write_u32(&mut self, access: Access<Physical>, data: u32) -> Result<(), Exception> {
        self.write_u32_bytes(access, data, 4)
    }

    pub fn write_u32_bytes(&mut self, mut access: Access<Physical>, data: u32, len: usize) -> Result<(), Exception> {
        match self.mapping(&mut access)? {
            Uart => self.uart.write_bytes(access, len, &data.to_le_bytes())?,
            Ram  => self.ram.write_bytes(access, len, &data.to_le_bytes())?,
        }
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
    fn read_byte(&self, mut access: Access<Physical>) -> Result<u8, Exception> {
        match self.mapping(&mut access)? {
            Uart => self.uart.read_byte(access),
            Ram  => self.ram.read_byte(access),
        }
    }

    fn write_byte(&mut self, mut access: Access<Physical>, data: u8) -> Result<(), Exception> {
        match self.mapping(&mut access)? {
            Uart => self.uart.write_byte(access, data),
            Ram  => self.ram.write_byte(access, data),
        }
    }

    fn read_bytes(&self, mut access: Access<Physical>, size: usize, des: &mut [u8]) -> Result<(), Exception> {
        match self.mapping(&mut access)? {
            Uart => self.uart.read_bytes(access, size, des),
            Ram  => self.ram.read_bytes(access, size, des),
        }
    }

    fn write_bytes(&mut self, mut access: Access<Physical>, size: usize, src: &[u8]) -> Result<(), Exception> {
        match self.mapping(&mut access)? {
            Uart => self.uart.write_bytes(access, size, src),
            Ram  => self.ram.write_bytes(access, size, src),
        }
    }
}
