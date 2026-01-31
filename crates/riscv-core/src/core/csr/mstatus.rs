use modular_bitfield::prelude::*;

#[bitfield]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mstatus {
    #[skip] __: B1, // uie: need N extension
    pub sie: B1,
    #[skip] __: B1, // Reserved
    pub mie: B1, 
    #[skip] __: B1, // upie: need N extension
    pub spie: B1, 
    #[skip] __: B1, // User Big Endian: Always 0
    pub mpie: B1,
    pub spp: B1,
    #[skip] __: B2, // Vector Status: need V extension
    pub mpp: B2,
    pub fs: B2, 
    pub xs: B2, 
    pub mprv: B1,
    pub sum: B1,
    pub mxr: B1,
    pub tvm: B1,
    pub tw: B1,
    pub tsr: B1,
    #[skip] __: B8, // WPRI: Reserved
    pub sd: B1,
}

const M_MODE_WRITE_MASK: u32 = 
    (1 << 1)  | (1 << 3)  | (1 << 5)  | (1 << 7)  | (1 << 8)  | (3 << 11) | (3 << 13) | (3 << 15) | 
    (1 << 16) | (1 << 17) | (1 << 18) | (1 << 16) | (1 << 17) | (1 << 18) | (1 << 19) | (1 << 20) | (1 << 21);

const S_MODE_WRITE_MASK: u32 = 
    (1 << 1) | (1 << 5) | (1 << 8) | 
    (3 << 13) | (3 << 15) | (1 << 18) | (1 << 19);

const S_MODE_READ_MASK: u32 = S_MODE_WRITE_MASK | (1 << 31);

impl Mstatus {
    pub fn read_m(&self) -> u32{
        u32::from(*self)
    }

    pub fn read_s(&self) -> u32{
        u32::from(*self) & S_MODE_READ_MASK
    }

    pub fn write_m(&mut self, data: u32) {
        *self = ((u32::from(*self) & !M_MODE_WRITE_MASK) | (data & M_MODE_WRITE_MASK)).into();
        self.check_update_sd();
    }

    pub fn write_s(&mut self, data: u32) {
        *self = ((u32::from(*self) & !S_MODE_WRITE_MASK) | (data & S_MODE_WRITE_MASK)).into();
        self.check_update_sd();
    }

    fn check_update_sd(&mut self) {
        if self.fs() == 0b11 || self.xs() == 0b11 {
            self.set_sd(1);
        } else {
            self.set_sd(0);
        }
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