//! Program Counter define and implment for Risc-V

use super::Reset; 

/// Program Counter structure. Has a pointer that every step plus 4 (bytes)
#[derive(Debug)]
pub struct PC {
    pointer: u32,
}

impl PC {
    pub fn step(&mut self) {
        self.pointer += 4;
    }

    pub fn get(&self) -> u32 {
        self.pointer
    }

    pub fn reset(&mut self) {
        self.pointer = 0;
    }

    pub fn related_addressing(&mut self, offset: i32) {
        self.pointer = self.pointer.wrapping_add_signed(offset << 1);
    }

    pub fn directed_addressing(&mut self, address: u32) {
        self.pointer = address;
    }
}

impl Default for PC {
    fn default() -> Self {
        PC{pointer: 0}
    }
}

impl Reset for PC {
    fn reset(&mut self) {
        self.pointer = 0;
    }
}