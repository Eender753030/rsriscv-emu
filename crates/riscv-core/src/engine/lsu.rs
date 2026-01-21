use crate::device::bus::SystemBus;
use crate::exception::Exception;

pub struct Lsu;

impl Lsu {
    pub fn load(bus: &mut SystemBus, src: u32, offset: i32, num: usize) -> Result<u32, Exception> {
        let addr = src.wrapping_add_signed(offset);
        bus.read_u32_bytes(addr, num, false)
    }

    pub fn load_signed(bus: &mut SystemBus, src: u32, offset: i32, num: usize) -> Result<u32, Exception> {
        let addr = src.wrapping_add_signed(offset);
        bus.read_u32_bytes(addr, num, true)
    }

    pub fn store(bus: &mut SystemBus, des: u32, src: u32, offset: i32, num: usize) -> Result<(), Exception> {
        let addr = des.wrapping_add_signed(offset);
        bus.write_u32_bytes(addr, src, num)
    }
}
