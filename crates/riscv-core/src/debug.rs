#[cfg(feature = "s")]
use crate::core::PrivilegeMode;

#[derive(Debug, Clone, PartialEq)]
pub struct MachineInfo {
    pub dram_size: usize,
    pub dram_base: u32,
    pub page_size: usize,
    #[cfg(feature = "s")]
    pub hit_rate: f32,
    #[cfg(feature = "s")]
    pub curr_mode: String,
}

impl MachineInfo {
    pub fn new(dram_size: usize, dram_base: u32 , page_size: usize, 
        #[cfg(feature = "s")] hit: usize, 
        #[cfg(feature = "s")] miss: usize, 
        #[cfg(feature = "s")] mode: PrivilegeMode) -> Self {
            #[cfg(feature = "s")]
        let hit_rate = if hit + miss == 0 {
            f32::NAN
        } else {
            (hit as f32) / ((hit + miss) as f32)
        };

        #[cfg(feature = "s")]
        let curr_mode = match mode {
            PrivilegeMode::Machine    => "Machine",
            PrivilegeMode::Supervisor => "Surervisor",
            PrivilegeMode::User       => "User"
        }.to_string();

        MachineInfo { dram_size, dram_base, page_size, 
            #[cfg(feature = "s")] hit_rate, 
            #[cfg(feature = "s")] curr_mode}
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