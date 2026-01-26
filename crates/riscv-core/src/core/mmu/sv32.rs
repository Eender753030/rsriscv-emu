use modular_bitfield::prelude::*;

#[bitfield]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sv32Vpn {
    pub offset: B12,
    pub vpn_0: B10,
    pub vpn_1: B10,
}

impl From<u32> for Sv32Vpn {
    fn from(value: u32) -> Self {
        Self::from_bytes(value.to_le_bytes())
    }
}

#[bitfield]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sv32Pte {
    v: B1,
    r: B1,
    w: B1,
    x: B1,
    u: B1,
    #[skip] __: B1,
    a: B1,
    d: B1,
    #[skip] __: B2,
    pub ppn: B22,
}

impl From<u32> for Sv32Pte {
    fn from(value: u32) -> Self {
        Self::from_bytes(value.to_le_bytes())
    }
}

impl From<Sv32Pte> for u32 {
    fn from(value: Sv32Pte) -> Self {
        Self::from_le_bytes(value.into_bytes())
    }
}

impl Sv32Pte {
    pub fn is_valid(&self) -> bool {
        self.v() > 0
    }

    pub fn is_access_zero_and_set(&mut self) -> bool {
        if self.a() == 0 {
            self.set_a(1);
            true
        } else {
            false
        }
    }

    pub fn is_dirty_zero_and_set(&mut self) -> bool {
        if self.d() == 0 {
            self.set_d(1);
            true
        } else {
            false
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.can_write() | self.can_read() | self.can_execute() 
    }

    pub fn can_write(&self) -> bool {
        self.w() > 0
    }

    pub fn can_read(&self) -> bool {
        self.r() > 0
    }

    pub fn can_execute(&self) -> bool {
        self.x() > 0
    }

    pub fn can_user(&self) -> bool {
        self.u() > 0
    }
}