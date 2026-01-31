mod sv32;
mod tlb;

use crate::Result;
use crate::core::CsrFile;
use crate::core::privilege::PrivilegeMode;
use crate::device::bus::SystemBus;
use crate::core::access::{Access, AccessType, Physical, Virtual};

use sv32::{Sv32Pte, Sv32Vpn};
use tlb::{Tlb, TlbResult};

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
        csrs: &CsrFile,
        bus: &mut SystemBus,
    ) -> Result<Access<Physical>> {
        let v_addr = access.addr; 

        if mode == PrivilegeMode::Machine {
            return Ok(access.bypass());
        }

        let (asid, root_ppn) = match csrs.check_satp(mode)? {
            Some((asid, ppn)) => (asid, ppn),
            None => return Ok(access.bypass()),
        };

        let tlb_res = self.tlb.lookup(csrs, v_addr, asid, access.kind, mode);

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

        Self::access_check(&pte, &access, mode, csrs)?;

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

    fn pte_walk(vpn: u16, ppn: u32, access: &Access<Virtual>, bus: &mut SystemBus) -> Result<(Sv32Pte, u32, bool)> {
        let pte_addr = (ppn << 12) + (vpn * 4) as u32;

        let pte_access = Access::new(pte_addr, AccessType::Load);

        let pte = Sv32Pte::from(bus.read_u32(pte_access)?);

        if !pte.is_valid() || (!pte.can_read() && pte.can_write()) {
            return  Err(access.into_page_exception())
        } 

        Ok((pte, pte_addr, pte.is_leaf()))
    }

    fn access_check(pte: &Sv32Pte, access: &Access<Virtual>, mode: PrivilegeMode, csrs: &CsrFile) -> Result<()> {
        let can_access = match access.kind {
            AccessType::Load  => pte.can_read() || (pte.can_execute() && csrs.check_mxr()),
            AccessType::Store => pte.can_write(),
            AccessType::Fetch => pte.can_execute(),
            #[cfg(feature = "a")]
            AccessType::Amo => pte.can_read() && pte.can_write(),
        };
        
        let can_mode = 
            (mode == PrivilegeMode::User       && pte.can_user()) ||
            (mode == PrivilegeMode::Supervisor && (csrs.check_sum() || !pte.can_user()));
    
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
mod tests;
