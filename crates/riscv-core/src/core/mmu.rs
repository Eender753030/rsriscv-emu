mod sv32;

pub mod access;

use crate::Exception;
use crate::core::privilege::PrivilegeMode;
use crate::device::bus::SystemBus;

use sv32::{Sv32Pte, Sv32Vpn};
use access::{Access, AccessType, Physical, Virtual};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Mmu;

impl Mmu {
    pub fn translate(access: Access<Virtual>, mode: PrivilegeMode, ppn_opt: Option<u32>, bus: &mut SystemBus) -> Result<Access<Physical>, Exception> {
        let v_addr = access.addr; 
        Ok(match mode {
            PrivilegeMode::Machine => access.bypass(),
            PrivilegeMode::Supervisor | PrivilegeMode::User => {
                if let Some(ppn) = ppn_opt {
                    let vpn = Sv32Vpn::from(v_addr);
                    let (pte1, pte1_addr, is_leaf) = Self::pte_walk(
                        vpn.vpn_1() as u32, ppn, &access, bus)?;

                    let pte0_opt = if is_leaf {
                        None
                    } else {
                        let (pte0, pte0_addr, is_leaf) = Self::pte_walk(
                            vpn.vpn_0() as u32, pte1.ppn(), &access, bus)?;
                        if !is_leaf {
                            return Err(access.to_page_exception());
                        }
                        Some((pte0, pte0_addr))
                    };

                    let (mut leaf_pte, leaf_pte_addr) = if let Some((pte0, addr)) = pte0_opt {
                        (pte0, addr)
                    } else {
                        (pte1, pte1_addr)
                    };

                    Self::access_check(&leaf_pte, access, mode)?;

                    let leaf_pte_access = Access::new(leaf_pte_addr, access.kind);
                    if leaf_pte.is_access_zero_and_set() {
                        bus.write_u32(leaf_pte_access, leaf_pte.into())?;
                    }
                    if access.kind == AccessType::Store && leaf_pte.is_dirty_zero_and_set() {
                        bus.write_u32(leaf_pte_access, leaf_pte.into())?;
                    }
                    
                    let p_addr = if pte0_opt.is_some() {
                        (leaf_pte.ppn() << 12) | vpn.offset() as u32
                    } else {
                        let ppn_0 = leaf_pte.ppn() & 0x3ff;
                        if ppn_0 != 0 {
                            return Err(access.to_page_exception());
                        }
                        let ppn_1 = leaf_pte.ppn() & 0x3ffc00;
                        ppn_1 << 12 | (vpn.vpn_0() as u32) << 12 | vpn.offset() as u32
                    };
                    access.into_physical(p_addr)
                } else {
                    access.bypass()
                }
            }
        })
    }

    fn pte_walk(vpn: u32, ppn: u32, access: &Access, bus: &mut SystemBus) -> Result<(Sv32Pte, u32, bool), Exception> {
        let pte_addr = (ppn << 12) + vpn * 4;

        let pte_access = Access::new(pte_addr, AccessType::Load);

        let pte = Sv32Pte::from(bus.read_u32(pte_access)?);

        Ok(if !pte.is_valid() || (!pte.can_read() && pte.can_write()) {
            return  Err(access.to_page_exception())
        } else if pte.is_leaf() {
            (pte, pte_addr, true)
        } else {
            (pte, pte_addr, false)
        })
    }

    fn access_check(pte: &Sv32Pte, access: Access, mode: PrivilegeMode) -> Result<(), Exception> {
        let can_access = match access.kind {
            AccessType::Load  => !pte.can_read(),
            AccessType::Store => !pte.can_write(),
            AccessType::Fetch => !pte.can_execute(),
        };
        
        let can_mode = 
            mode == PrivilegeMode::User       && !pte.can_user() ||
            mode == PrivilegeMode::Supervisor && pte.can_user();
         
        if can_access && can_mode {
            Ok(())
        } else {
            Err(access.to_page_exception())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Exception;
    use crate::core::Mmu;
    use crate::core::privilege::PrivilegeMode;
    use crate::device::bus::SystemBus;
    use crate::core::mmu::access::{Access, AccessType};
    
    fn make_pte(ppn: u32, v: bool, r: bool, w: bool, x: bool, u: bool, a: bool, d: bool) -> u32 {
        let mut pte = 0;
        if v { pte |= 1 << 0; }
        if r { pte |= 1 << 1; }
        if w { pte |= 1 << 2; }
        if x { pte |= 1 << 3; }
        if u { pte |= 1 << 4; }
        if a { pte |= 1 << 6; }
        if d { pte |= 1 << 7; }
        pte |= (ppn & 0x3FFFFF) << 10;
        pte
    }

    fn write_pte(bus: &mut SystemBus, addr: u32, pte: u32) {
        let access = Access::new(addr, AccessType::Store);
        bus.write_u32(access.into_physical(addr), pte).expect("Setup PTE failed");
    }

    fn read_ram_u32(bus: &mut SystemBus, addr: u32) -> u32 {
        let access = Access::new(addr, AccessType::Load);
        bus.read_u32(access.into_physical(addr)).expect("Read RAM failed")
    }

    #[test]
    fn test_bypass_mode() {
        let mut bus = SystemBus::default();
        let va = 0x8000_5555;
        let access = Access::new(va, AccessType::Load);

        let res = Mmu::translate(access, PrivilegeMode::Machine, Some(0x80001), &mut bus);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().addr, va, "M-Mode should bypass MMU");

        let res = Mmu::translate(access, PrivilegeMode::Supervisor, None, &mut bus);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().addr, va, "Bare mode should bypass MMU");
    }

    #[test]
    fn test_sv32_4k_page_translation_and_accessed_bit() {
        let mut bus = SystemBus::default();
        
        // Root Page Table (L1): 0x8000_1000 (PPN: 0x80001)
        // Leaf Page Table (L0): 0x8000_2000 (PPN: 0x80002)
        // Data Page           : 0x8000_3000 (PPN: 0x80003)
        // Virtual Address     : 0x8000_0000 (VPN1=0x200, VPN0=0x000, Offset=0)
        
        let root_ppn = 0x80001;
        let leaf_pt_ppn = 0x80002;
        let target_ppn = 0x80003;
        
        let va = 0x8000_0000;
        let vpn1 = (va >> 22) & 0x3FF;
        let vpn0 = (va >> 12) & 0x3FF;

        let root_pte_addr = (root_ppn << 12) + vpn1 * 4;
        let root_pte_val = make_pte(leaf_pt_ppn, true, false, false, false, false, false, false); // V=1, RWX=0 (Pointer)
        write_pte(&mut bus, root_pte_addr, root_pte_val);

        let leaf_pte_addr = (leaf_pt_ppn << 12) + vpn0 * 4;
        let leaf_pte_val = make_pte(target_ppn, true, true, true, false, false, false, false);
        write_pte(&mut bus, leaf_pte_addr, leaf_pte_val);

        let access = Access::new(va, AccessType::Load);
        let res = Mmu::translate(access, PrivilegeMode::Supervisor, Some(root_ppn), &mut bus);

        assert!(res.is_ok(), "Translation failed: {:?}", res.err());
        let pa = res.unwrap().addr;
        assert_eq!(pa, target_ppn << 12, "Physical address mismatch");

        let updated_leaf_pte = read_ram_u32(&mut bus, leaf_pte_addr);
        assert_eq!(updated_leaf_pte & (1 << 6), (1 << 6), "Accessed bit should be set by MMU");
    }

    #[test]
    fn test_sv32_megapage_translation() {
        let mut bus = SystemBus::default();
        let root_ppn = 0x80001;
        let target_megapage_ppn = 0x80400; 

        let va = 0x8040_0abc; 
        let vpn1 = (va >> 22) & 0x3FF;

        let root_pte_addr = (root_ppn << 12) + vpn1 * 4;

        let root_pte_val = make_pte(target_megapage_ppn, true, true, true, false, false, false, false);
        write_pte(&mut bus, root_pte_addr, root_pte_val);

        let access = Access::new(va, AccessType::Store);
        let res = Mmu::translate(access, PrivilegeMode::Supervisor, Some(root_ppn), &mut bus);

        assert!(res.is_ok());
        let pa = res.unwrap().addr;
        
        let expected_pa = (target_megapage_ppn << 12) | (va & 0x3FFFFF);
        assert_eq!(pa, expected_pa);

        let updated_pte = read_ram_u32(&mut bus, root_pte_addr);
        assert_eq!(updated_pte & (1 << 6), (1 << 6), "A bit missing");
        assert_eq!(updated_pte & (1 << 7), (1 << 7), "D bit missing");
    }

    #[test]
    fn test_page_fault_read_only() {
        let mut bus = SystemBus::default();
        let root_ppn = 0x80001;
        let target_ppn = 0x80000; 
        let va = 0x8000_0000;

        let vpn1 = (va >> 22) & 0x3FF;
        let root_pte_addr = (root_ppn << 12) + vpn1 * 4;
 
        let pte_val = make_pte(target_ppn, true, true, false, false, false, true, false); 
        write_pte(&mut bus, root_pte_addr, pte_val);

        let load_access = Access::new(va, AccessType::Load);
        assert!(Mmu::translate(load_access, PrivilegeMode::Supervisor, Some(root_ppn), &mut bus).is_ok());

        let store_access = Access::new(va, AccessType::Store);
        let res = Mmu::translate(store_access, PrivilegeMode::Supervisor, Some(root_ppn), &mut bus);
        
        match res {
            Err(Exception::StoreOrAmoPageFault(addr)) => assert_eq!(addr, va),
            _ => panic!("Should throw StoreOrAmoPageFault, got {:?}", res),
        }
    }

    #[test]
    fn test_page_fault_invalid() {
        let mut bus = SystemBus::default();
        let root_ppn = 0x80001;
        let va = 0xDEAD_BEEF;

        let vpn1 = (va >> 22) & 0x3FF;
        let pte_addr = (root_ppn << 12) + vpn1 * 4;
        
        let access_init = Access::new(pte_addr, AccessType::Store);
        bus.write_u32(access_init.into_physical(pte_addr), 0).expect("Init PT memory failed");
        
        let access = Access::new(va, AccessType::Load);
        let res = Mmu::translate(access, PrivilegeMode::Supervisor, Some(root_ppn), &mut bus);

        match res {
            Err(Exception::LoadPageFault(addr)) => assert_eq!(addr, va),
            _ => panic!("Should throw LoadPageFault for invalid PTE, got {:?}", res),
        }
    }
}
