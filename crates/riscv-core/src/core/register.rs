#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RegisterFile {
    reg: [u32; 32],
}

impl RegisterFile {
    pub fn read(&mut self, id: u8) -> u32 {
        self.reg[id as usize]
    }

    pub fn write(&mut self, id: u8, data: u32) {
        if id == 0 {
            return;
        }
        self.reg[id as usize] = data;
    }

    pub fn reset(&mut self) {
        self.reg.fill(0);
    }

    pub fn iter(&self) -> IteratorRegisterFile<'_> {
        IteratorRegisterFile { id: 0, reg: &self }
    }
}

impl std::ops::Index<u8> for RegisterFile {
    type Output = u32;
    fn index(&self, index: u8) -> &Self::Output {
        &self.reg[index as usize]
    }
}

pub struct IteratorRegisterFile<'a> {
    id: u8,
    reg: &'a RegisterFile,
}

impl <'a>Iterator for IteratorRegisterFile<'a> {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        if self.id < 32 {
            let next = Some(self.reg[self.id]);
            self.id += 1;
            next
        } else {
            None
        }
    }
}