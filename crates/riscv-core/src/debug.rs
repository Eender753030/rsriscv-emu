#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MachineInfo {
    dram_size: usize,
    dram_base: u32,
    page_size: usize,
}

impl MachineInfo {
    pub fn new(dram_size: usize, dram_base: u32 , page_size: usize) -> Self {
        MachineInfo { dram_size, dram_base, page_size }
    }

    pub fn get_info(&self) -> (usize, u32, usize) {
        (self.dram_size, self.dram_base, self.page_size)
    }
}

pub trait DebugInterface {
    fn inspect_regs(&self) -> [u32; 32];

    fn inspect_pc(&self) -> u32;

    fn inspect_csrs(&self) -> Vec<(String, u32)>;

    fn inspect_mem(&self, start: u32, len: usize) -> Vec<u8>;

    fn get_info(&self) -> MachineInfo;
}