use modular_bitfield::prelude::*;

#[bitfield]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Satp {
    pub ppn: B22,
    pub asid: B9,
    pub mode: B1,
}

impl From<Satp> for u32 {
    fn from(value: Satp) -> Self {
        Self::from_le_bytes(value.into_bytes())
    }
}

impl From<u32> for Satp {
    fn from(value: u32) -> Self {
        Self::from_bytes(value.to_le_bytes())
    }
}

impl Default for Satp {
    fn default() -> Self {
        Self::new()
    }
}