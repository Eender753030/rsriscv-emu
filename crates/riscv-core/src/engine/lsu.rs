use crate::core::{Access, AccessType, Mmu, PrivilegeMode};
use crate::exception::Exception;
use crate::device::bus::SystemBus;

pub struct Lsu;

impl Lsu {
    pub fn load(bus: &mut SystemBus, src: u32, offset: i32, num: usize, mode: PrivilegeMode, ppn_opt: Option<u32>) -> Result<u32, Exception> {
        let addr = src.wrapping_add_signed(offset);
        let va_access = Access::new(addr, AccessType::Load);
        let pa_access = Mmu::translate(va_access, mode, ppn_opt, bus)?;

        bus.read_u32_bytes(pa_access, num, false).or_else(|e| Err(match e {
            Exception::LoadAccessFault(_)  => Exception::LoadAccessFault(addr),
            Exception::StoreAccessFault(_) => Exception::StoreAccessFault(addr),
            _ => e,
        }))
    }

    pub fn load_signed(bus: &mut SystemBus, src: u32, offset: i32, num: usize, mode: PrivilegeMode, ppn_opt: Option<u32>) -> Result<u32, Exception> {
        let addr = src.wrapping_add_signed(offset);
        let va_access = Access::new(addr, AccessType::Load);
        let pa_access = Mmu::translate(va_access, mode, ppn_opt, bus)?;   
        
        bus.read_u32_bytes(pa_access, num, true).or_else(|e| Err(match e {
            Exception::LoadAccessFault(_)  => Exception::LoadAccessFault(addr),
            Exception::StoreAccessFault(_) => Exception::StoreAccessFault(addr),
            _ => e,
        }))
    }

    pub fn store(bus: &mut SystemBus, des: u32, src: u32, offset: i32, num: usize, mode: PrivilegeMode, ppn_opt: Option<u32>) -> Result<(), Exception> {
        let addr = des.wrapping_add_signed(offset);
        let va_access = Access::new(addr, AccessType::Store);
        let pa_access = Mmu::translate(va_access, mode, ppn_opt, bus)?;   
        
        bus.write_u32_bytes(pa_access, src, num).or_else(|e| Err(match e {
            Exception::LoadAccessFault(_)  => Exception::LoadAccessFault(addr),
            Exception::StoreAccessFault(_) => Exception::StoreAccessFault(addr),
            _ => e,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::Lsu;
    use crate::device::bus::{SystemBus, DRAM_BASE_ADDR};
    use crate::core::PrivilegeMode;

    #[test]
    fn test_store_load_word() {
        // 初始化 Bus (預設帶有 2GB RAM)
        let mut bus = SystemBus::default();
        let mode = PrivilegeMode::Machine;
        let addr = DRAM_BASE_ADDR;
        let val = 0xDEADBEEF;

        Lsu::store(&mut bus, addr, val, 0, 4, mode, None).expect("Store failed");
        
        let res = Lsu::load(&mut bus, addr, 0, 4, mode, None).expect("Load failed");
        assert_eq!(res, val, "Read back value mismatch");
    }

    #[test]
    fn test_offset_handling() {
        let mut bus = SystemBus::default();
        let mode = PrivilegeMode::Machine;
        
        let base = DRAM_BASE_ADDR + 0x100;
        let offset = -4; 
        let val = 0x12345678;

        Lsu::store(&mut bus, base, val, offset, 4, mode, None).unwrap();
        

        let res = Lsu::load(&mut bus, base, offset, 4, mode, None).unwrap();
        assert_eq!(res, val);
        
        let actual_addr = (base as i32 + offset) as u32;
        let direct_res = Lsu::load(&mut bus, actual_addr, 0, 4, mode, None).unwrap();
        assert_eq!(direct_res, val);
    }

    #[test]
    fn test_sign_extension() {
        let mut bus = SystemBus::default();
        let mode = PrivilegeMode::Machine;
        let addr = DRAM_BASE_ADDR + 0x20;
        

        Lsu::store(&mut bus, addr, 0xFF, 0, 1, mode, None).unwrap();

        let lbu = Lsu::load(&mut bus, addr, 0, 1, mode, None).unwrap();
        assert_eq!(lbu, 0x000000FF, "Lbu failed: expected zero extension");

        let lb = Lsu::load_signed(&mut bus, addr, 0, 1, mode, None).unwrap();
        assert_eq!(lb, 0xFFFFFFFF, "Lb failed: expected sign extension");
        
        Lsu::store(&mut bus, addr + 4, 0xFFAA, 0, 2, mode, None).unwrap();
        
        let lh = Lsu::load_signed(&mut bus, addr + 4, 0, 2, mode, None).unwrap();
        assert_eq!(lh, 0xFFFF_FFAA, "Lh failed");
        
        let lhu = Lsu::load(&mut bus, addr + 4, 0, 2, mode, None).unwrap();
        assert_eq!(lhu, 0x0000_FFAA, "Lhu failed");
    }
}