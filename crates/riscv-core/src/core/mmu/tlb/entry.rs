use modular_bitfield::prelude::*;

use crate::core::{CsrFile, PrivilegeMode}; 
use crate::core::access::AccessType;
use crate::core::mmu::sv32::Sv32Pte;

#[bitfield]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TlbEntry {
    pub valid: B1,
    pub global: B1,
    pub tag: B14,
    pub asid: B9,
    pub ppn: B22,
    pub r: B1,
    pub w: B1,
    pub x: B1,
    pub u: B1,
    pub a: B1,
    pub d: B1,
    pub page_size: B1,
    #[skip] __: B10,
}

impl TlbEntry {
    pub fn is_valid(&self) -> bool {
        self.valid() > 0
    }

    pub fn is_global(&self) -> bool {
        self.global() > 0
    }

    pub fn is_mega_page(&self) -> bool {
        self.page_size() > 0
    }

    pub fn can_read(&self) -> bool {
        self.r() > 0
    }

    pub fn can_write(&self) -> bool {
        self.w() > 0
    }

    pub fn can_execute(&self) -> bool {
        self.x() > 0
    }

    pub fn can_user(&self) -> bool {
        self.u() > 0
    }

    pub fn is_accessed(&self) -> bool {
        self.a() > 0
    }

    pub fn is_dirty(&self) -> bool {
        self.d() > 0
    }

    pub fn set_flags(&mut self, pte: Sv32Pte) {
        self.set_valid(pte.is_valid() as u8);
        self.set_r(pte.can_read() as u8);
        self.set_w(pte.can_write() as u8);
        self.set_x(pte.can_execute() as u8);
        self.set_u(pte.can_user() as u8);
        self.set_global(pte.is_global() as u8);
        self.set_a(pte.is_accessed() as u8);
        self.set_d(pte.is_dirty() as u8);
    } 

    pub fn access_check(&self, kind: AccessType, mode: PrivilegeMode, csrs: &CsrFile) -> bool {
        let can_access = match kind {
            AccessType::Load  => self.can_read() || (self.can_execute() && csrs.check_mxr()),
            AccessType::Store => self.can_write(),
            AccessType::Fetch => self.can_execute(),
            #[cfg(feature = "a")]
            AccessType::Amo   => self.can_read() && self.can_write(),
        };
        
        let can_mode = 
            (mode == PrivilegeMode::User       && self.can_user()) ||
            (mode == PrivilegeMode::Supervisor && (csrs.check_sum() || !self.can_user()));
    
        can_access && can_mode
    }

    pub fn ad_check(&self, kind: AccessType) -> bool {
        match kind {
            AccessType::Load | AccessType::Fetch => self.is_accessed(),
            AccessType::Store => self.is_accessed() && self.is_dirty(),
            #[cfg(feature = "a")]
            AccessType::Amo   => self.is_accessed() && self.is_dirty(),
        }
    }

    pub fn flush(&mut self) {
        self.set_valid(0);
    }
}