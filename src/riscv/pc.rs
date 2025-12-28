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
}