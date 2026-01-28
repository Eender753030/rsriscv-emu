mod sv32;
mod tlb;

pub mod access;

use crate::Exception;
use crate::core::privilege::PrivilegeMode;
use crate::device::bus::SystemBus;

use sv32::{Sv32Pte, Sv32Vpn};
use tlb::{Tlb, TlbResult};
use access::{Access, AccessType, Physical, Virtual};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Mmu {
    tlb: Tlb,
    hit_count: usize,
    miss_count: usize,
}

impl Mmu {
    pub fn translate(
        &mut self, 
        access: Access<Virtual>, 
        mode: PrivilegeMode, 
        satp_opt: Option<(u16, u32)>,
        bus: &mut SystemBus
    ) -> Result<Access<Physical>, Exception> {
        let v_addr = access.addr; 

        if mode == PrivilegeMode::Machine {
            return Ok(access.bypass());
        }

        let (asid, root_ppn) = match satp_opt {
            Some((asid, ppn)) => (asid, ppn),
            None => return Ok(access.bypass()),
        };

        let tlb_res = self.tlb.lookup(v_addr, asid, access.kind, mode);

        match tlb_res {
            TlbResult::Hit(is_mega, ppn) => {
                self.hit_count += 1;
                let p_addr = Self::get_physical(v_addr, ppn, is_mega);
                return Ok(access.into_physical(p_addr));
            },
            TlbResult::PageFault => {
                self.hit_count += 1;
                return Err(access.into_page_exception());
            },
            TlbResult::UpdateAD => self.hit_count += 1,
            TlbResult::Miss => self.miss_count += 1,
        }

        let vpn = Sv32Vpn::from(v_addr);

        let (mut pte, pte_addr, is_mega) = {
            let (pte1, addr1, is_leaf) = Self::pte_walk(
                vpn.vpn_1(), root_ppn, &access, bus
            )?;

            if is_leaf {
                (pte1, addr1, true)
            } else {
                let (pte0, addr0, is_leaf) = Self::pte_walk(
                    vpn.vpn_0(), pte1.ppn(), &access, bus
                )?;
                if !is_leaf {
                    return Err(access.into_page_exception());
                }
                (pte0, addr0, false)
            }
        };

        Self::access_check(&pte, &access, mode)?;

        let mut update_pte = false;

        if pte.is_access_zero_and_set() {
            update_pte = true;
        }
        if access.kind == AccessType::Store && pte.is_dirty_zero_and_set() {
            update_pte = true;
        }
        if update_pte {
            let pte_access = Access::new(pte_addr, access.kind);
            bus.write_u32(pte_access, pte.into())?;
        }

        if is_mega && (pte.ppn() & 0x3ff) != 0 {
            return Err(access.into_page_exception());
        }

        self.tlb.fill(v_addr, pte, asid, is_mega);

        let p_addr = Self::get_physical(v_addr, pte.ppn(), is_mega);

        Ok(access.into_physical(p_addr))
    }

    fn pte_walk(vpn: u16, ppn: u32, access: &Access<Virtual>, bus: &mut SystemBus) -> Result<(Sv32Pte, u32, bool), Exception> {
        let pte_addr = (ppn << 12) + (vpn * 4) as u32;

        let pte_access = Access::new(pte_addr, AccessType::Load);

        let pte = Sv32Pte::from(bus.read_u32(pte_access)?);

        if !pte.is_valid() || (!pte.can_read() && pte.can_write()) {
            return  Err(access.into_page_exception())
        } 

        Ok((pte, pte_addr, pte.is_leaf()))
    }

    fn access_check(pte: &Sv32Pte, access: &Access<Virtual>, mode: PrivilegeMode) -> Result<(), Exception> {
        let can_access = match access.kind {
            AccessType::Load  => pte.can_read(),
            AccessType::Store => pte.can_write(),
            AccessType::Fetch => pte.can_execute(),
        };
        
        let can_mode = 
            (mode == PrivilegeMode::User       && pte.can_user()) ||
            (mode == PrivilegeMode::Supervisor && !pte.can_user());
    
        if can_access && can_mode {
            Ok(())
        } else {
            Err(access.into_page_exception())
        }
    }

    fn get_physical(v_addr: u32, ppn: u32, is_mega: bool) -> u32 {
        let level_size = if is_mega {
                1 << 22
            } else {
                1 << 12
            };
        let offset_mask = level_size - 1;
        (ppn << 12) | (v_addr & offset_mask)
    }

    pub fn flush_tlb(&mut self, v_addr: u32, asid: u32) {
        let vpn = v_addr >> 12; 
        self.tlb.flush(vpn, asid as u16);
    }
}

#[cfg(test)]
mod tests {
    use crate::Exception;
    use crate::core::Mmu;
    use crate::core::mmu::{Sv32Pte};
    use crate::core::privilege::PrivilegeMode;
    use crate::device::bus::SystemBus;
    use crate::core::mmu::access::{Access, AccessType};
    use crate::core::mmu::tlb::TlbResult;
    
    fn make_pte(ppn: u32, 
        v: bool, 
        r: bool, 
        w: bool, 
        x: bool, 
        u: bool, 
        g: bool,
        a: bool, 
        d: bool
    ) -> u32 {
        let mut pte = 0;
        if v { pte |= 1 << 0; }
        if r { pte |= 1 << 1; }
        if w { pte |= 1 << 2; }
        if x { pte |= 1 << 3; }
        if u { pte |= 1 << 4; }
        if g { pte |= 1 << 5; }
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
        let mut mmu = Mmu::default();

        let mut bus = SystemBus::default();
        let va = 0x8000_5555;
        let access = Access::new(va, AccessType::Load);

        let res = mmu.translate(access, PrivilegeMode::Machine, Some((0, 0x80001)), &mut bus);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().addr, va, "M-Mode should bypass MMU");

        let res = mmu.translate(access, PrivilegeMode::Supervisor, None, &mut bus);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().addr, va, "Bare mode should bypass MMU");
    }

    #[test]
    fn test_sv32_4k_page_translation_and_accessed_bit() {
        let mut mmu = Mmu::default();
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
        let root_pte_val = make_pte(leaf_pt_ppn, true, false, false, false, false, false, false, false); // V=1, RWX=0 (Pointer)
        write_pte(&mut bus, root_pte_addr, root_pte_val);

        let leaf_pte_addr = (leaf_pt_ppn << 12) + vpn0 * 4;
        let leaf_pte_val = make_pte(target_ppn, true, true, true, false, false, false, false, false);
        write_pte(&mut bus, leaf_pte_addr, leaf_pte_val);

        let access = Access::new(va, AccessType::Load);
        let res = mmu.translate(access, PrivilegeMode::Supervisor, Some((0, root_ppn)), &mut bus);

        assert!(res.is_ok(), "Translation failed: {:?}", res.err());
        let pa = res.unwrap().addr;
        assert_eq!(pa, target_ppn << 12, "Physical address mismatch");

        let updated_leaf_pte = read_ram_u32(&mut bus, leaf_pte_addr);
        assert_eq!(updated_leaf_pte & (1 << 6), (1 << 6), "Accessed bit should be set by MMU");
    }

    #[test]
    fn test_sv32_megapage_translation() {
        let mut mmu = Mmu::default();
        let mut bus = SystemBus::default();
        let root_ppn = 0x80001;
        let target_megapage_ppn = 0x80400; 

        let va = 0x8040_0abc; 
        let vpn1 = (va >> 22) & 0x3FF;

        let root_pte_addr = (root_ppn << 12) + vpn1 * 4;

        let root_pte_val = make_pte(target_megapage_ppn, true, true, true, false, false, false, false, false);
        write_pte(&mut bus, root_pte_addr, root_pte_val);

        let access = Access::new(va, AccessType::Store);
        let res = mmu.translate(access, PrivilegeMode::Supervisor, Some((0, root_ppn)), &mut bus);

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
        let mut mmu = Mmu::default();
        let mut bus = SystemBus::default();
        let root_ppn = 0x80001;
        let target_ppn = 0x80000; 
        let va = 0x8000_0000;

        let vpn1 = (va >> 22) & 0x3FF;
        let root_pte_addr = (root_ppn << 12) + vpn1 * 4;
 
        let pte_val = make_pte(target_ppn, true, true, false, false, false, false, true, false); 
        write_pte(&mut bus, root_pte_addr, pte_val);

        let load_access = Access::new(va, AccessType::Load);
        assert!(mmu.translate(load_access, PrivilegeMode::Supervisor, Some((0, root_ppn)), &mut bus).is_ok());

        let store_access = Access::new(va, AccessType::Store);
        let res = mmu.translate(store_access, PrivilegeMode::Supervisor, Some((0, root_ppn)), &mut bus);
        
        match res {
            Err(Exception::StoreOrAmoPageFault(addr)) => assert_eq!(addr, va),
            _ => panic!("Should throw StoreOrAmoPageFault, got {:?}", res),
        }
    }

    #[test]
    fn test_page_fault_invalid() {
        let mut mmu = Mmu::default();
        let mut bus = SystemBus::default();
        let root_ppn = 0x80001;
        let va = 0xDEAD_BEEF;

        let vpn1 = (va >> 22) & 0x3FF;
        let pte_addr = (root_ppn << 12) + vpn1 * 4;
        
        let access_init = Access::new(pte_addr, AccessType::Store);
        bus.write_u32(access_init.into_physical(pte_addr), 0).expect("Init PT memory failed");
        
        let access = Access::new(va, AccessType::Load);
        let res = mmu.translate(access, PrivilegeMode::Supervisor, Some((0, root_ppn)), &mut bus);

        match res {
            Err(Exception::LoadPageFault(addr)) => assert_eq!(addr, va),
            _ => panic!("Should throw LoadPageFault for invalid PTE, got {:?}", res),
        }
    }

    #[test]
    fn test_tlb() {
        let mut mmu = Mmu::default();
        let mut bus = SystemBus::default();

        let root_ppn = 0x80001;
        let leaf_pt_ppn = 0x80002;
        let target_ppn = 0x80003;
        let va = 0x8000_0000;

        let vpn1 = (va >> 22) & 0x3FF;
        let vpn0 = (va >> 12) & 0x3FF;

        let root_pte_addr = (root_ppn << 12) + vpn1 * 4;
        write_pte(&mut bus, root_pte_addr, make_pte(leaf_pt_ppn, true, false, false, false, false, false, false, false));

        let leaf_pte_addr = (leaf_pt_ppn << 12) + vpn0 * 4;
        write_pte(&mut bus, leaf_pte_addr, make_pte(target_ppn, true, true, true, false, false, false, false, false));

        let access = Access::new(va, AccessType::Load);
        let res = mmu.translate(access, PrivilegeMode::Supervisor, Some((0, root_ppn)), &mut bus);
        assert!(res.is_ok());
        assert_eq!(mmu.hit_count, 0, "First access should be a miss");
        assert_eq!(mmu.miss_count, 1, "First access should increment miss count");
    
        write_pte(&mut bus, leaf_pte_addr, 0);

        let access2 = Access::new(va, AccessType::Load);
        let res2 = mmu.translate(access2, PrivilegeMode::Supervisor, Some((0, root_ppn)), &mut bus);
        
        assert!(res2.is_ok(), "Should hit TLB and ignore invalid memory PTE");
        assert_eq!(res2.unwrap().addr, target_ppn << 12);
        assert_eq!(mmu.hit_count, 1, "Second access should be a hit");
        assert_eq!(mmu.miss_count, 1, "Miss count should not increase");
    }

    #[test]
    fn test_flush_tlb() {
        let mut mmu = Mmu::default();

        // 1. VPN=0x10, ASID=1, Global=0
        // 2. VPN=0x20, ASID=1, Global=1
        // 3. VPN=0x10, ASID=2, Global=0 (Same VPN as A)
        let ppn_a = 0xABC;
        let ppn_b = 0xDEF;
        let ppn_c = 0x123;

        // Entry 1
        let pte_a = make_pte(ppn_a, true, true, true, false, true, false, true, true); // V, R, W, U
        mmu.tlb.fill(0x10 << 12, Sv32Pte::from(pte_a), 1, false);

        // Entry 2
        let pte_b = make_pte(ppn_b, true, true, false, false, false, true, true, true); 
        mmu.tlb.fill(0x20 << 12, Sv32Pte::from(pte_b), 1, false);

        // Entry 3
        let pte_c = make_pte(ppn_c, true, true, true, false, true, false, true, true);
        mmu.tlb.fill(0x10 << 12, Sv32Pte::from(pte_c), 2, false);

        mmu.flush_tlb(0x10 << 12, 1);

        // Check 1: lookup should Miss
        let res1 = mmu.tlb.lookup(0x10 << 12, 1, AccessType::Load, PrivilegeMode::User);
        assert!(matches!(res1, TlbResult::Miss));

        // Check 2: Global should Hit (Case 4 Protection)
        let res2 = mmu.tlb.lookup(0x20 << 12, 1, AccessType::Load, PrivilegeMode::Supervisor);
        assert!(matches!(res2, TlbResult::Hit(_, _)));

        // Check 3: ASID 2 should Hit
        let res3 = mmu.tlb.lookup(0x10 << 12, 2, AccessType::Load, PrivilegeMode::User);
        assert!(matches!(res3, TlbResult::Hit(_, _)));

        // --- Refill Entry 1 for next test ---
        mmu.tlb.fill(0x10 << 12, Sv32Pte::from(pte_a), 1, false);

        mmu.flush_tlb(0, 1); // vaddr=0 triggers ASID flush

        let res1 = mmu.tlb.lookup(0x10 << 12, 1, AccessType::Load, PrivilegeMode::User);
        assert!(matches!(res1, TlbResult::Miss));

        let res2 = mmu.tlb.lookup(0x20 << 12, 1, AccessType::Load, PrivilegeMode::Supervisor);
        assert!(matches!(res2, TlbResult::Hit(_, _)));

        mmu.tlb.fill(0x10 << 12, Sv32Pte::from(pte_a), 1, false); // Refill A
        mmu.flush_tlb(0x10 << 12, 0); // asid=0 triggers VAddr flush

        let res1 = mmu.tlb.lookup(0x10 << 12, 1, AccessType::Load, PrivilegeMode::User);
        assert!(matches!(res1, TlbResult::Miss));
        let res3 = mmu.tlb.lookup(0x10 << 12, 2, AccessType::Load, PrivilegeMode::User);
        assert!(matches!(res3, TlbResult::Miss));


        // Flush All
        mmu.tlb.fill(0x10 << 12, Sv32Pte::from(pte_a), 1, false);
        mmu.flush_tlb(0, 0);

        let res2 = mmu.tlb.lookup(0x20 << 12, 1, AccessType::Load, PrivilegeMode::Supervisor);
        assert!(matches!(res2, TlbResult::Miss));
    }
}
