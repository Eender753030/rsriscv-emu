#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MachineInfo {
    pub dram_size: usize,
    pub dram_base: u32,
    pub page_size: usize,
    pub hit_rate: f32,
}

impl MachineInfo {
    pub fn new(dram_size: usize, dram_base: u32 , page_size: usize, hit: usize, miss: usize) -> Self {
        let hit_rate = if hit + miss == 0 {
            f32::NAN
        } else {
            (hit as f32) / ((hit + miss) as f32)
        };
        MachineInfo { dram_size, dram_base, page_size, hit_rate }
    }
}

pub trait DebugInterface {
    fn inspect_regs(&self) -> [u32; 32];

    fn inspect_pc(&self) -> u32;

    #[cfg(feature = "zicsr")]
    fn inspect_csrs(&self) -> Vec<(String, u32)>;

    fn inspect_bus(&self, start: u32, len: usize) -> Vec<u8>;

    fn get_info(&self) -> MachineInfo;
}