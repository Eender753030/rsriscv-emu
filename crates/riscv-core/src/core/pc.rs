use crate::constance::DRAM_BASE_ADDR;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PC {
    pointer: u32,
}

impl PC {
    pub fn step(&mut self) {
        self.pointer += 4;
    }

    #[cfg(feature = "c")]
    pub fn half_step(&mut self) {
        self.pointer += 2;
    }

    pub fn get(&self) -> u32 {
        self.pointer
    }

    pub fn set(&mut self, pointer: u32) {
        self.pointer = pointer;
    }

    pub fn reset(&mut self) {
        self.pointer = DRAM_BASE_ADDR;
    }

    pub fn related_addressing(&mut self, offset: i32) {
        self.pointer = self.pointer.wrapping_add_signed(offset);
    }

    pub fn directed_addressing(&mut self, address: u32) {
        self.pointer = address & !1;
    }
}

impl Default for PC {
    fn default() -> Self {
        PC { pointer: DRAM_BASE_ADDR }
    }
}
