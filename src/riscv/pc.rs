#[derive(Debug)]
pub struct PC {
    pointer: u32,
}

impl PC {
    pub fn new() -> Self {
        PC{pointer: 0}
    }

    pub fn step(&mut self) {
        self.pointer += 4;
    }

    pub fn get(&self) -> u32 {
        self.pointer
    }

    pub fn related_addressing(&mut self, offset: i32) {
        self.pointer = self.pointer.wrapping_add_signed(offset << 1);
    }

    pub fn directed_addressing(&mut self, address: u32) {
        self.pointer = address;
    }
}