const INIT_RAM_START: u32 = 0x8000_0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        self.pointer = INIT_RAM_START;
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
        PC { pointer: INIT_RAM_START }
    }
}
