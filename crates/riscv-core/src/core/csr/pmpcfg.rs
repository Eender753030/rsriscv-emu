use modular_bitfield::prelude::*;

use crate::core::{Access, AccessType, Physical, PrivilegeMode};

use MatchingMode::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MatchingMode {
    #[default]
    Off   = 0b00,
    Tor   = 0b01,
    Na4   = 0b10, 
    Napot = 0b11,
}

#[bitfield]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PmpEntry {
    r: B1,
    w: B1,
    x: B1,
    a: B2,
    #[skip] __: B2,
    l: B1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Pmpcfg {
    pmp: [PmpEntry; 4]
}

impl PmpEntry {
    pub fn mode(&self) -> MatchingMode {
        match self.a() {
            0b00 => Off,
            0b01 => Tor,
            0b10 => Na4,
            0b11 => Napot,
            _    => unreachable!("pmpcfg's 'a' only has 2 bits"),
        }
    }

    // `true` is pass
    pub fn mode_check(&self, mode: PrivilegeMode) -> bool {
        match mode {
            PrivilegeMode::Machine => self.l() == 0,
            _                      => false,
        }
    }

    // `true` is pass
    pub fn access_check(&self, access: Access<Physical>) -> bool {
        match access.kind {
            AccessType::Load  => self.r() > 0,
            AccessType::Store => self.w() > 0,
            AccessType::Fetch => self.x() > 0,
        }
    }
}

impl From<PmpEntry> for u8 {
    fn from(value: PmpEntry) -> Self {
        value.into_bytes()[0]       
    }
}

impl From<u8> for PmpEntry {
    fn from(value: u8) -> Self {
        PmpEntry::from_bytes([value; 1])
    }
}

impl std::ops::Index<usize> for Pmpcfg {
    type Output = PmpEntry;
    fn index(&self, index: usize) -> &Self::Output {
        &self.pmp[index]
    }
}

impl From<Pmpcfg> for u32 {
    fn from(value: Pmpcfg) -> Self {
        let bytes: [u8 ;4] = [
            value.pmp[0].into(),
            value.pmp[1].into(),
            value.pmp[2].into(),
            value.pmp[3].into(),
        ];
        u32::from_le_bytes(bytes)
    }
}

impl From<u32> for Pmpcfg {
    fn from(value: u32) -> Self {
        let bytes = value.to_le_bytes();
        let pmp: [PmpEntry; 4] = [
            bytes[0].into(),
            bytes[1].into(),
            bytes[2].into(),
            bytes[3].into(),
        ];
        Pmpcfg { pmp }
    }
}
