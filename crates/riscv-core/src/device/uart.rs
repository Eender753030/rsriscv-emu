use std::io::{self, Write};

use crate::core::{Access, Physical};
use crate::device::Device;
use crate::Exception;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Uart {
    // thr: u32,
    // lsr: u32,
}

impl Device for Uart {
    fn read_byte(&self, assess: Access<Physical>) -> Result<u8, Exception> {
        Ok(match assess.addr {
            0x05 => 0x20,
            _    => 0,
        })
    }

    fn write_byte(&mut self, assess: Access<Physical>, data: u8) -> Result<(), Exception> {
        if assess.addr == 0x00 { 
            print!("{}", data as char);
   
            io::stdout().flush().unwrap();
        }
        Ok(())
    }

    fn read_bytes(&self, assess: Access<Physical>, size: usize, des: &mut [u8]) -> Result<(), Exception> {
        if size > 0 {
            des[0] = self.read_byte(assess)?;
        }
        Ok(())
    }

    fn write_bytes(&mut self, assess: Access<Physical>, size: usize, src: &[u8]) -> Result<(), Exception> {
        if size > 0 {
            self.write_byte(assess, src[0])?;
        }
        Ok(())
    }
}