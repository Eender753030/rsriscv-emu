use modular_bitfield::prelude::*;

#[bitfield]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mstatus {
    #[skip] __: B1, 
    pub sie: B1, // bit 1
    #[skip] __: B1,
    pub mie: B1, // bit 3
    #[skip] __: B1,
    pub spie: B1, // bit 5
    #[skip] __: B1,
    pub mpie: B1, // bit 7
    pub spp: B1, // bit 8
    #[skip] __: B2,
    pub mpp: B2, // bit 12:11
    #[skip] __: B19,
}

const M_MODE_MASK: u32 = (1 << 1) | (1 << 3) | (1 << 5) | (3 << 7) | (3 << 11);
const S_MODE_MASK: u32 = (1 << 1) | (1 << 5) | (1 << 8);

impl Mstatus {
    pub fn read_m(&self) -> u32{
        u32::from(*self)
    }

    pub fn read_s(&self) -> u32{
        u32::from(*self) & S_MODE_MASK
    }

    pub fn write_m(&mut self, data: u32) {
        *self = ((u32::from(*self) & !M_MODE_MASK) | (data & M_MODE_MASK)).into();
    }

    pub fn write_s(&mut self, data: u32) {
        *self = ((u32::from(*self) & !S_MODE_MASK) | (data & S_MODE_MASK)).into();
    }
}

impl From<Mstatus> for u32 {
    fn from(value: Mstatus) -> Self {
        Self::from_le_bytes(value.into_bytes())
    }
}

impl From<u32> for Mstatus {
    fn from(value: u32) -> Self {
        Self::from_bytes(value.to_le_bytes())
    }
}

impl Default for Mstatus {
    fn default() -> Self {
        Self::new()
    }
}