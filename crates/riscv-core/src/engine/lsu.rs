use crate::core::{Access, AccessType, CsrFile, Mmu, PrivilegeMode};
use crate::exception::Exception;
use crate::device::bus::SystemBus;

pub struct Lsu;

impl Lsu {
    pub fn load(
        bus: &mut SystemBus, 
        csrs: &CsrFile,
        src: u32, 
        offset: i32, 
        num: usize, 
        mode: PrivilegeMode, 
    ) -> Result<u32, Exception> {
        let addr = src.wrapping_add_signed(offset);
        let va_access = Access::new(addr, AccessType::Load);
        let pa_access = Mmu::translate(va_access, mode, csrs.check_satp(), bus)?;
        
        csrs.pmp_check(pa_access, num, mode).or_else(|e| Err(match e {
            Exception::LoadAccessFault(_)  => Exception::LoadAccessFault(addr),
            _ => e,
        }))?;

        bus.read_u32_bytes(pa_access, num, false).or_else(|e| Err(match e {
            Exception::LoadAccessFault(_)  => Exception::LoadAccessFault(addr),
            _ => e,
        }))
    }

    pub fn load_signed(
        bus: &mut SystemBus, 
        csrs: &CsrFile,
        src: u32, 
        offset: i32, 
        num: usize,
        mode: PrivilegeMode, 
    ) -> Result<u32, Exception> {
        let addr = src.wrapping_add_signed(offset);
        let va_access = Access::new(addr, AccessType::Load);
        let pa_access = Mmu::translate(va_access, mode, csrs.check_satp(), bus)?;   
        
        csrs.pmp_check(pa_access, num, mode).or_else(|e| Err(match e {
            Exception::LoadAccessFault(_)  => Exception::LoadAccessFault(addr),
            _ => e,
        }))?;

        bus.read_u32_bytes(pa_access, num, true).or_else(|e| Err(match e {
            Exception::LoadAccessFault(_)  => Exception::LoadAccessFault(addr),
            _ => e,
        }))
    }

    pub fn store(
        bus: &mut SystemBus, 
        csrs: &CsrFile,
        des: u32, 
        src: u32, 
        offset: i32, 
        num: usize, 
        mode: PrivilegeMode, 
    ) -> Result<(), Exception> {
        let addr = des.wrapping_add_signed(offset);
        let va_access = Access::new(addr, AccessType::Store);
        let pa_access = Mmu::translate(va_access, mode, csrs.check_satp(), bus)?;   
        
        csrs.pmp_check(pa_access, num, mode).or_else(|e| Err(match e {
            Exception::StoreAccessFault(_) => Exception::StoreAccessFault(addr),
            _ => e,
        }))?;

        bus.write_u32_bytes(pa_access, src, num).or_else(|e| Err(match e {
            Exception::StoreAccessFault(_) => Exception::StoreAccessFault(addr),
            _ => e,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::Lsu;
    use crate::device::bus::{SystemBus, DRAM_BASE_ADDR};
    use crate::core::{CsrFile, PrivilegeMode};

    #[test]
    fn test_store_load_word() {
        let mut bus = SystemBus::default();
        let csr = CsrFile::default();
        let mode = PrivilegeMode::Machine;
        let addr = DRAM_BASE_ADDR;
        let val = 0xDEADBEEF;

        Lsu::store(&mut bus, &csr, addr, val, 0, 4, mode).expect("Store failed");
        
        let res = Lsu::load(&mut bus, &csr, addr, 0, 4, mode).expect("Load failed");
        assert_eq!(res, val, "Read back value mismatch");
    }

    #[test]
    fn test_offset_handling() {
        let mut bus = SystemBus::default();
        let mode = PrivilegeMode::Machine;
        let csr = CsrFile::default();
        
        let base = DRAM_BASE_ADDR + 0x100;
        let offset = -4; 
        let val = 0x12345678;

        Lsu::store(&mut bus, &csr, base, val, offset, 4, mode).unwrap();
        

        let res = Lsu::load(&mut bus, &csr, base, offset, 4, mode).unwrap();
        assert_eq!(res, val);
        
        let actual_addr = (base as i32 + offset) as u32;
        let direct_res = Lsu::load(&mut bus, &csr, actual_addr, 0, 4, mode).unwrap();
        assert_eq!(direct_res, val);
    }

    #[test]
    fn test_sign_extension() {
        let mut bus = SystemBus::default();
        let mode = PrivilegeMode::Machine;
        let addr = DRAM_BASE_ADDR + 0x20;
        let csr = CsrFile::default();

        Lsu::store(&mut bus, &csr, addr, 0xFF, 0, 1, mode).unwrap();

        let lbu = Lsu::load(&mut bus, &csr, addr, 0, 1, mode).unwrap();
        assert_eq!(lbu, 0x000000FF, "Lbu failed: expected zero extension");

        let lb = Lsu::load_signed(&mut bus, &csr, addr, 0, 1, mode).unwrap();
        assert_eq!(lb, 0xFFFFFFFF, "Lb failed: expected sign extension");
        
        Lsu::store(&mut bus, &csr, addr + 4, 0xFFAA, 0, 2, mode).unwrap();
        
        let lh = Lsu::load_signed(&mut bus, &csr, addr + 4, 0, 2, mode).unwrap();
        assert_eq!(lh, 0xFFFF_FFAA, "Lh failed");
        
        let lhu = Lsu::load(&mut bus, &csr, addr + 4, 0, 2, mode).unwrap();
        assert_eq!(lhu, 0x0000_FFAA, "Lhu failed");
    }
}