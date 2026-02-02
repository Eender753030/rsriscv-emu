mod entry;
mod plru;
mod result;
mod set;

use std::ops::IndexMut;

use crate::core::{CsrFile, PrivilegeMode}; 
use crate::core::access::AccessType;
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
    pub fn lookup(
        &mut self, 
        csrs: &CsrFile,
        v_addr: u32,
        asid: u16, 
        kind: AccessType, 
        mode: PrivilegeMode) -> TlbResult {
        let vpn = v_addr >> 12;
        let set_idx = Self::get_set_idx(vpn);
        let tag = (vpn >> TLB_SET_SHIFT) as u16;
        
        let target = self.find(set_idx, tag, asid);
        
        match target {
            Some(idx) => {
                self.sets[set_idx].update(idx);
                let entry = &self.sets[set_idx].entries[idx];

                if entry.access_check(kind, mode, csrs) {
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

    pub fn flush(&mut self, vpn: u32, asid: u16) {
        match (vpn, asid) {
            (0, 0) => self.flush_all(),
            (0, a) => self.flush_by_asid(a),
            (v, 0) => self.flush_by_address(v),
            (v, a) => self.flush_by_both(v, a),
        }
    }

    fn flush_all(&mut self) {
        self.sets.iter_mut().for_each(|set| set.flush());
    }

    fn flush_by_asid(&mut self, asid: u16) {
        self.sets.iter_mut().for_each(|set| {
            set.entries.iter_mut().for_each(|entry| {
                if entry.asid() == asid 
                    && entry.is_valid() 
                    && !entry.is_global() {
                    entry.flush();
                }
            })
        });
    }

    fn flush_by_address(&mut self, vpn: u32) {
        let set_index = Self::get_set_idx(vpn);
        let tag = (vpn >> TLB_SET_SHIFT) as u16;

        self.sets[set_index].entries.iter_mut()
            .for_each(|entry| {
                if entry.tag() == tag && entry.is_valid() {
                    entry.flush();
                }
            }); 
    }

    fn flush_by_both(&mut self, vpn: u32, asid: u16) {
        let set_index = Self::get_set_idx(vpn);
        let tag = (vpn >> TLB_SET_SHIFT) as u16;

        self.sets[set_index].entries.iter_mut()
            .for_each(|entry| {
               if   entry.tag() == tag 
                    && entry.asid() == asid
                    && entry.is_valid()
                    && !entry.is_global() {
                    entry.flush();
                }
            }); 
    }

    pub fn reset(&mut self) {
        self.flush_all();
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