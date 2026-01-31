use crate::{Exception, Result};
#[cfg(feature = "zicsr")]
use crate::core::{CsrFile, PrivilegeMode};
#[cfg(feature = "s")]
use crate::core::Mmu;
use crate::core::access::{Access, AccessType, Physical, Virtual};
use crate::device::bus::SystemBus;

#[derive(Debug, PartialEq, Eq)]
pub struct Lsu<'a> {
    bus: &'a mut SystemBus,
    #[cfg(feature = "s")] mmu: &'a mut Mmu,
    #[cfg(feature = "zicsr")] csrs: &'a CsrFile,
    #[cfg(feature = "zicsr")] mode: PrivilegeMode, 
}

impl<'a> Lsu<'a> {
    pub fn new(
        bus: &'a mut SystemBus, 
        #[cfg(feature = "s")] mmu: &'a mut Mmu, 
        #[cfg(feature = "zicsr")] csrs: &'a CsrFile, 
        #[cfg(feature = "zicsr")] mode: PrivilegeMode
    ) -> Self {
        Self { 
            bus,
            #[cfg(feature = "s")] mmu,  
            #[cfg(feature = "zicsr")] csrs, 
            #[cfg(feature = "zicsr")] mode 
        }
    }

    pub fn load(&mut self, src: u32, offset: i32, num: usize) -> Result<u32> {
        let addr = src.wrapping_add_signed(offset);
        let va_access = Access::new(addr, AccessType::Load);
        let pa_access = self.pre_work(va_access, num)?;

        self.bus.read_u32_bytes(pa_access, num, false).map_err(|e| match e {
            Exception::LoadAccessFault(_)  => Exception::LoadAccessFault(addr),
            _ => e,
        })
    }

    pub fn load_signed(&mut self, src: u32, offset: i32, num: usize) -> Result<u32> {
        let addr = src.wrapping_add_signed(offset);
        let va_access = Access::new(addr, AccessType::Load);
        let pa_access = self.pre_work(va_access, num)?;

        self.bus.read_u32_bytes(pa_access, num, true).map_err(|e| match e {
            Exception::LoadAccessFault(_)  => Exception::LoadAccessFault(addr),
            _ => e,
        })
    }

    #[cfg(feature = "a")]
    pub fn atomic_load(&mut self, src: u32) -> Result<(u32, u32)> {
        let addr = src;
        if addr & 0b11 != 0 {
            return Err(Exception::LoadAddressMisaligned);
        }

        let va_access = Access::new(addr, AccessType::Load);
        let pa_access = self.pre_work(va_access, 4)?;

        let res = self.bus.read_u32_bytes(pa_access, 4, false).map_err(|e| match e {
            Exception::LoadAccessFault(_)  => Exception::LoadAccessFault(addr),
            _ => e,
        })?;
        Ok((res, pa_access.addr))
    }

    pub fn store(&mut self, des: u32, src: u32, offset: i32, num: usize,
        #[cfg(feature = "a")] reservation: &mut Option<u32>) -> Result<()> {
        let addr = des.wrapping_add_signed(offset);
        let va_access = Access::new(addr, AccessType::Store);
        let pa_access = self.pre_work(va_access, num)?;

        #[cfg(feature = "a")]
        if let Some(addr) = *reservation && addr == pa_access.addr {
            *reservation = None;
        }


        self.bus.write_u32_bytes(pa_access, src, num).map_err(|e| match e {
            Exception::StoreOrAmoAccessFault(_) => Exception::StoreOrAmoAccessFault(addr),
            _ => e,
        })
    }

    #[cfg(feature = "a")]
    pub fn atomic_store(&mut self, des: u32, src: u32, reservation: &mut Option<u32>) -> Result<bool> {
        let addr = des;
        if addr & 0b11 != 0 {
            return Err(Exception::LoadAddressMisaligned);
        }
        let va_access = Access::new(addr, AccessType::Store);
        let pa_access = self.pre_work(va_access, 4)?;

        match reservation {
            Some(addr) => if *addr != pa_access.addr {
                return Ok(false);
            },
            None => return Ok(false),
        }

        *reservation = None;

        self.bus.write_u32_bytes(pa_access, src, 4).map_err(|e| match e {
            Exception::StoreOrAmoAccessFault(_) => Exception::StoreOrAmoAccessFault(addr),
            _ => e,
        })?;
        Ok(true)
    }

    #[cfg(feature = "a")]
    pub fn atomic_operate<F>(&mut self, des: u32, data: u32, ope: F, reservation: &mut Option<u32>) -> Result<u32> 
        where F: Fn(u32, u32) -> u32
    {
        let addr = des;
        if addr & 0b11 != 0 {
            return Err(Exception::LoadAddressMisaligned);
        }
        let va_access = Access::new(addr, AccessType::Amo);
        let pa_access = self.pre_work(va_access, 4)?;

        if let Some(addr) = *reservation && addr == pa_access.addr {
            *reservation = None;
        }

        let tmp = self.bus.read_u32_bytes(pa_access, 4, false).map_err(|e| match e {
            Exception::StoreOrAmoAccessFault(_)  => Exception::StoreOrAmoAccessFault(addr),
            _ => e,
        })?;

        let res_data = ope(tmp, data);

        self.bus.write_u32_bytes(pa_access, res_data, 4).map_err(|e| match e {
            Exception::StoreOrAmoAccessFault(_) => Exception::StoreOrAmoAccessFault(addr),
            _ => e,
        })?;

        Ok(tmp)
    }

    #[allow(unused_variables)]
    fn pre_work(&mut self, va_access: Access<Virtual>, num: usize) -> Result<Access<Physical>> { 
        #[cfg(not(feature = "s"))]
        let pa_access = va_access.bypass();

        #[cfg(feature = "s")]
            let pa_access = self.mmu.translate(
                va_access, self.mode, self.csrs, self.bus
            )?;   
            
        #[cfg(feature = "zicsr")] {
            self.csrs.pmp_check(pa_access, num, self.mode).map_err(|e| match e {
                Exception::LoadAccessFault(_)  => Exception::LoadAccessFault(va_access.addr),
                Exception::StoreOrAmoAccessFault(_) => Exception::StoreOrAmoAccessFault(va_access.addr),
                _ => e,
            })?;
        }
        Ok(pa_access)      
    }
}

#[cfg(test)]
mod tests {
    use super::Lsu;
    use crate::device::bus::{SystemBus, DRAM_BASE_ADDR};
    #[cfg(feature = "zicsr")]
    use crate::core::{CsrFile, PrivilegeMode};
    #[cfg(feature = "s")]
    use crate::core::Mmu;

    #[test]
    fn test_store_load_word() {
        let mut bus = SystemBus::default();
        #[cfg(feature = "s")]
        let mut mmu = Mmu::default();
        #[cfg(feature = "zicsr")]
        let csrs = CsrFile::default();
        #[cfg(feature = "zicsr")]
        let mode = PrivilegeMode::Machine;
        let mut lsu = Lsu::new(
            &mut bus,
            #[cfg(feature = "s")] &mut mmu,
            #[cfg(feature = "zicsr")] &csrs, 
            #[cfg(feature = "zicsr")] mode
        );
        let addr = DRAM_BASE_ADDR;
        let val = 0xDEADBEEF;

        lsu.store(addr, val, 0, 4,
            #[cfg(feature = "a")]&mut None).expect("Store failed");
        
        let res = lsu.load(addr, 0, 4).expect("Load failed");
        assert_eq!(res, val, "Read back value mismatch");
    }

    #[test]
    fn test_offset_handling() {
        let mut bus = SystemBus::default();
        #[cfg(feature = "s")]
        let mut mmu = Mmu::default();
        #[cfg(feature = "zicsr")]
        let csrs = CsrFile::default();
        #[cfg(feature = "zicsr")]
        let mode = PrivilegeMode::Machine;

        let mut lsu = Lsu::new(
            &mut bus,
            #[cfg(feature = "s")] &mut mmu,
            #[cfg(feature = "zicsr")] &csrs, 
            #[cfg(feature = "zicsr")] mode
        );
        let base = DRAM_BASE_ADDR + 0x100;
        let offset = -4; 
        let val = 0x12345678;

        lsu.store(base, val, offset, 4,
            #[cfg(feature = "a")]&mut None).unwrap();
        
        let res = lsu.load(base, offset, 4).unwrap();
        assert_eq!(res, val);
        
        let actual_addr = (base as i32 + offset) as u32;
        let direct_res = lsu.load(actual_addr, 0, 4).unwrap();
        assert_eq!(direct_res, val);
    }

    #[test]
    fn test_sign_extension() {
        let mut bus = SystemBus::default();
        let addr = DRAM_BASE_ADDR + 0x20;     
        #[cfg(feature = "s")]
        let mut mmu = Mmu::default();
        #[cfg(feature = "zicsr")]
        let csrs = CsrFile::default();
        #[cfg(feature = "zicsr")]
        let mode = PrivilegeMode::Machine;

        let mut lsu = Lsu::new(
            &mut bus,
            #[cfg(feature = "s")] &mut mmu,
            #[cfg(feature = "zicsr")] &csrs, 
            #[cfg(feature = "zicsr")] mode
        );

        lsu.store(addr, 0xFF, 0, 1,
            #[cfg(feature = "a")]&mut None).unwrap();

        let lbu = lsu.load(addr, 0, 1).unwrap();
        assert_eq!(lbu, 0x000000FF, "Lbu failed: expected zero extension");

        let lb = lsu.load_signed(addr, 0, 1).unwrap();
        assert_eq!(lb, 0xFFFFFFFF, "Lb failed: expected sign extension");
        
        lsu.store(addr + 4, 0xFFAA, 0, 2,
            #[cfg(feature = "a")]&mut None).unwrap();
        
        let lh = lsu.load_signed(addr + 4, 0, 2).unwrap();
        assert_eq!(lh, 0xFFFF_FFAA, "Lh failed");
        
        let lhu = lsu.load(addr + 4, 0, 2).unwrap();
        assert_eq!(lhu, 0x0000_FFAA, "Lhu failed");
    }
}