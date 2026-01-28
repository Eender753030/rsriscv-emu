mod entry;
mod plru;
mod result;
mod set;

use std::ops::IndexMut;

use crate::core::{AccessType, PrivilegeMode}; 
use crate::core::mmu::sv32::Sv32Pte;

use entry::TlbEntry;
use set::TlbSet;

pub use result::TlbResult;

const TLB_WAY_NUM: usize = 4;
const TLB_SET_SHIFT: usize = 6;
const TLB_SET_NUM: usize = 1 << 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tlb {
    sets: [TlbSet; TLB_SET_NUM],
}

impl Tlb {
    pub fn lookup(&mut self, v_addr: u32, asid: u16, kind: AccessType, mode: PrivilegeMode) -> TlbResult {
        let vpn = v_addr >> 12;
        let set_idx = Self::get_set_idx(vpn);
        let tag = (vpn >> TLB_SET_SHIFT) as u16;
        
        let target = self.find(set_idx, tag, asid);
        
        match target {
            Some(idx) => {
                self.sets[set_idx].update(idx);
                let entry = &self.sets[set_idx].entries[idx];

                if entry.access_check(kind, mode) {
                    if entry.ad_check(kind) {
                        TlbResult::Hit(entry.is_mega_page(), entry.ppn())
                    } else {
                        TlbResult::UpdateAD
                    }
                } else {
                    TlbResult::PageFault
                }
            },
            None => TlbResult::Miss,
        }
    }

    pub fn fill(&mut self, v_addr: u32, pte: Sv32Pte, asid: u16, is_mega: bool) {
        let vpn = v_addr >> 12;
        let set_idx = Self::get_set_idx(vpn);
        let tag = (vpn >> TLB_SET_SHIFT) as u16;

        let in_tlb = self.find(set_idx, tag, asid);

        let (idx, victim) = if let Some(i) = in_tlb {
            (i, self.sets[set_idx].entries.index_mut(i))
        } else {
            self.sets[set_idx].get_victim()
        };

        victim.set_tag(tag);
        victim.set_flags(pte);
        victim.set_ppn(pte.ppn());
        victim.set_asid(asid);
        victim.set_page_size(is_mega as u8);
        
        self.sets[set_idx].plru.update(idx);
    }

    fn find(&self, set_idx: usize, tag: u16, asid: u16) -> Option<usize> {
        for (i, entry) in self.sets[set_idx].entries.iter().enumerate() {
            if entry.is_valid() 
                && entry.tag() == tag
                && (entry.is_global() || entry.asid() == asid) {
                return Some(i);
            }
        }
        None
    }

    fn get_set_idx(vpn: u32) -> usize {
        vpn as usize & (TLB_SET_NUM - 1)
    }
}

impl Default for TlbEntry {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Tlb {
    fn default() -> Self {
        Self { sets: [TlbSet::default(); TLB_SET_NUM] }
    }
}