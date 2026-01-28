use modular_bitfield::prelude::*;

#[bitfield]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlruState{
    b0: B1,
    b1: B1,
    b2: B1,
    #[skip] __: B5,
}

impl PlruState {
    pub fn get_victim(&self) -> usize {
        if self.b0() == 0 {
            if self.b1() == 0 { 
                0 
            } else { 
                1 
            }
        } else if self.b2() == 0 {
            2
        } else { 
            3 
        }
    }

    pub fn update(&mut self, way: usize) {
        match way {
            0 => {
                self.set_b0(1);
                self.set_b1(1);
            },
            1 => {
                self.set_b0(1);
                self.set_b1(0);
            },
            2 => {
                self.set_b0(0);
                self.set_b2(1);
            },
            3 => {
                self.set_b0(0);
                self.set_b2(0);
            },
            _ => unreachable!("There are only 4-ways for TLB"),
        }
    }

    pub fn flush(&mut self) {
        self.set_b0(0);
        self.set_b1(0);
        self.set_b2(0);
    }
}
