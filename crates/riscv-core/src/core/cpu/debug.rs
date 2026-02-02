use crate::debug::{DebugInterface, MachineInfo};
use crate::core::access::{Access, AccessType};
use crate::core::cpu::Cpu;
use crate::device::Device;

impl DebugInterface for Cpu {
    fn inspect_regs(&self) -> [u32; 32] {
        self.regs.inspect()
    }

    fn inspect_pc(&self) -> u32 {
        self.pc.get()
    }

    #[cfg(feature = "zicsr")]
    fn inspect_csrs(&self) -> Vec<(String, u32)> {
        self.csrs.inspect()
    }

    fn inspect_bus(&self, addr: u32, len: usize) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![0; len]; 
        let access = Access::new(addr, AccessType::Load);
        let _ = self.bus.read_bytes(access, len, &mut bytes);
        bytes
    }    

    fn get_info(&self) -> MachineInfo {
        let (dram_size, dram_base, page_size) = self.bus.ram_info();
        let dram_size = dram_size / 1024 / 1024 / 1024;
        let page_size = page_size / 1024;
        #[cfg(feature = "s")]
        let hit = self.mmu.hit_count;
        #[cfg(feature = "s")]
        let miss = self.mmu.miss_count;

        MachineInfo::new(dram_size, dram_base, page_size, 
            #[cfg(feature = "s")]hit, 
            #[cfg(feature = "s")]miss, 
            #[cfg(feature = "s")]self.mode)
    }
}