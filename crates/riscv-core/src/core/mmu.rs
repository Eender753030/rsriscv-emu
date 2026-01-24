pub mod access;
mod sv32;

use crate::{Exception, core::{mmu::sv32::Sv32Vpn, privilege::PrivilegeMode}, device::bus::SystemBus};

use sv32::Sv32Pte;
use access::{Access, AccessType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Mmu;

impl Mmu {
    pub fn translate(access: Access, mode: PrivilegeMode, ppn_opt: Option<u32>, bus: &mut SystemBus) -> Result<Access, Exception> {
        let v_addr = access.addr; 
        match mode {
            PrivilegeMode::Machine => Ok(access),
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
                    Ok(Access::new(p_addr, access.kind))
                } else {
                    Ok(access)
                }
            }
        }
    }

    fn pte_walk(vpn: u32, ppn: u32, access: &Access, bus: &mut SystemBus) -> Result<(Sv32Pte, u32, bool), Exception> {
        let pte_addr = (ppn << 12) + vpn * 4;

        let pte_access = Access::new(pte_addr, AccessType::Load);

        let pte = Sv32Pte::from(bus.read_u32(pte_access)?);

        if !pte.is_valid() || (!pte.can_read() && pte.can_write()) {
            Err(access.to_page_exception())
        } else if pte.is_leaf() {
                Ok((pte, pte_addr, true))
        } else {
            Ok((pte, pte_addr, false))
        }
    }

    fn access_check(pte: &Sv32Pte, access: Access, mode: PrivilegeMode) -> Result<(), Exception> {
        if match access.kind {
            AccessType::Load => !pte.can_read(),
            AccessType::Store => !pte.can_write(),
            AccessType::Fetch => !pte.can_execute(),
        } || (mode == PrivilegeMode::User && !pte.can_user() ||
                mode == PrivilegeMode::Supervisor && pte.can_user()) {
            Err(access.to_page_exception())
        } else {
            Ok(())
        }
    }
}
