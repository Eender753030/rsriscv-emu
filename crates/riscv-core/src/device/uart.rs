use crate::device::Device;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Uart {
    // thr: u32,
    // lsr: u32,
}

impl Device for Uart {
    fn read_byte(&self, addr: u32) -> Result<u8, crate::prelude::Exception> {
        match addr {
            0x05 => Ok(0x20),
            _ => Ok(0),
        }
    }

    fn write_byte(&mut self, addr: u32, data: u8) -> Result<(), crate::prelude::Exception> {
        match addr {
            0x00 => {
                print!("{}", data as char);

                use std::io::Write;
                std::io::stdout().flush().unwrap();
            }
            _ => {}
        }
        Ok(())
    }

    fn read_bytes(&self, addr: u32, size: usize, des: &mut [u8]) -> Result<(), crate::prelude::Exception> {
        if size > 0 {
            des[0] = self.read_byte(addr)?;
        }
        Ok(())
    }

    fn write_bytes(&mut self, addr: u32, size: usize, src: &[u8]) -> Result<(), crate::prelude::Exception> {
        if size > 0 {
            self.write_byte(addr, src[0])?;
        }
        Ok(())
    }
}