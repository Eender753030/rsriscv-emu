use crate::core::{Access, AccessType, CsrFile, PrivilegeMode};
use crate::core::{Mmu, Physical, Virtual};
use crate::exception::Exception;
use crate::device::bus::SystemBus;

#[derive(Debug, PartialEq, Eq)]
pub struct Lsu<'a> {
    mmu: &'a mut Mmu,
    bus: &'a mut SystemBus,
    csrs: &'a CsrFile,
    mode: PrivilegeMode, 
}

impl<'a> Lsu<'a> {
    pub fn new(mmu: &'a mut Mmu, bus: &'a mut SystemBus, csrs: &'a CsrFile, mode: PrivilegeMode) -> Self {
        Self { mmu, bus, csrs, mode }
    }

    pub fn load(&mut self, src: u32, offset: i32, num: usize) -> Result<u32, Exception> {
        let addr = src.wrapping_add_signed(offset);
        let va_access = Access::new(addr, AccessType::Load);
        let pa_access = self.pre_work(va_access, num)?;

        self.bus.read_u32_bytes(pa_access, num, false).map_err(|e| match e {
            Exception::LoadAccessFault(_)  => Exception::LoadAccessFault(addr),
            _ => e,
        })
    }

    pub fn load_signed(&mut self, src: u32, offset: i32, num: usize) -> Result<u32, Exception> {
        let addr = src.wrapping_add_signed(offset);
        let va_access = Access::new(addr, AccessType::Load);
        let pa_access = self.pre_work(va_access, num)?;

        self.bus.read_u32_bytes(pa_access, num, true).map_err(|e| match e {
            Exception::LoadAccessFault(_)  => Exception::LoadAccessFault(addr),
            _ => e,
        })
    }

    pub fn store(&mut self, des: u32, src: u32, offset: i32, num: usize) -> Result<(), Exception> {
        let addr = des.wrapping_add_signed(offset);
        let va_access = Access::new(addr, AccessType::Store);
        let pa_access = self.pre_work(va_access, num)?;

        self.bus.write_u32_bytes(pa_access, src, num).map_err(|e| match e {
            Exception::StoreAccessFault(_) => Exception::StoreAccessFault(addr),
            _ => e,
        })
    }

    fn pre_work(&mut self, va_access: Access<Virtual>, num: usize) -> Result<Access<Physical>, Exception> {
        let pa_access = self.mmu.translate(
            va_access, self.mode, self.csrs.check_satp(), self.bus
        )?;   
        
        self.csrs.pmp_check(pa_access, num, self.mode).map_err(|e| match e {
            Exception::LoadAccessFault(_)  => Exception::LoadAccessFault(va_access.addr),
            Exception::StoreAccessFault(_) => Exception::StoreAccessFault(va_access.addr),
            _ => e,
        })?;

        Ok(pa_access)
    }
}

#[cfg(test)]
mod tests {
    use super::Lsu;
    use crate::device::bus::{SystemBus, DRAM_BASE_ADDR};
    use crate::core::{CsrFile, Mmu, PrivilegeMode};

    #[test]
    fn test_store_load_word() {
        let mut mmu = Mmu::default();
        let mut bus = SystemBus::default();
        let csrs = CsrFile::default();
        let mode = PrivilegeMode::Machine;
        let mut lsu = Lsu::new(&mut mmu, &mut bus, &csrs, mode);
        let addr = DRAM_BASE_ADDR;
        let val = 0xDEADBEEF;

        lsu.store(addr, val, 0, 4).expect("Store failed");
        
        let res = lsu.load(addr, 0, 4).expect("Load failed");
        assert_eq!(res, val, "Read back value mismatch");
    }

    #[test]
    fn test_offset_handling() {
        let mut mmu = Mmu::default();
        let mut bus = SystemBus::default();
        let mode = PrivilegeMode::Machine;
        let csrs = CsrFile::default();
        let mut lsu = Lsu::new(&mut mmu, &mut bus, &csrs, mode);
        let base = DRAM_BASE_ADDR + 0x100;
        let offset = -4; 
        let val = 0x12345678;

        lsu.store(base, val, offset, 4).unwrap();
        
        let res = lsu.load(base, offset, 4).unwrap();
        assert_eq!(res, val);
        
        let actual_addr = (base as i32 + offset) as u32;
        let direct_res = lsu.load(actual_addr, 0, 4).unwrap();
        assert_eq!(direct_res, val);
    }

    #[test]
    fn test_sign_extension() {
        let mut mmu = Mmu::default();
        let mut bus = SystemBus::default();
        let mode = PrivilegeMode::Machine;
        let addr = DRAM_BASE_ADDR + 0x20;
        let csrs = CsrFile::default();
        let mut lsu = Lsu::new(&mut mmu, &mut bus, &csrs, mode);

        lsu.store(addr, 0xFF, 0, 1).unwrap();

        let lbu = lsu.load(addr, 0, 1).unwrap();
        assert_eq!(lbu, 0x000000FF, "Lbu failed: expected zero extension");

        let lb = lsu.load_signed(addr, 0, 1).unwrap();
        assert_eq!(lb, 0xFFFFFFFF, "Lb failed: expected sign extension");
        
        lsu.store(addr + 4, 0xFFAA, 0, 2).unwrap();
        
        let lh = lsu.load_signed(addr + 4, 0, 2).unwrap();
        assert_eq!(lh, 0xFFFF_FFAA, "Lh failed");
        
        let lhu = lsu.load(addr + 4, 0, 2).unwrap();
        assert_eq!(lhu, 0x0000_FFAA, "Lhu failed");
    }
}