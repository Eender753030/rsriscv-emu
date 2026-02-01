use riscv_core::Exception;
use riscv_core::debug::DebugInterface;

use crate::state::list_state::ListStateRecord;

#[derive(Debug, PartialEq, Eq)]
pub struct MachineSnapshot {
    pub ins: ListStateRecord<(u32, String)>,
    pub reg: ListStateRecord<u32>,
    #[cfg(feature = "zicsr")]
    pub csr: ListStateRecord<(String, u32)>,
    pub pc: u32,
    pub except: String,
}

impl MachineSnapshot {
    pub fn new<D: DebugInterface>(mach: &D, ins_list: Vec<(u32, String)>) -> Self {
        let ins = ListStateRecord::new(ins_list);
        let reg = ListStateRecord::new(mach.inspect_regs().into_iter().collect());
        #[cfg(feature = "zicsr")]
        let csr = ListStateRecord::new(mach.inspect_csrs());

        let except = "".to_string();
        let pc = mach.inspect_pc();

        MachineSnapshot { ins, reg, csr, pc, except }
    }

    pub fn update_snapshot<D: DebugInterface>(&mut self, mach: &D) {
        self.reg.list = mach.inspect_regs().into_iter().collect();
        #[cfg(feature = "zicsr")] {
        self.csr.list = mach.inspect_csrs();
        }
        self.pc = mach.inspect_pc();
    }

    pub fn update_exception(&mut self, except: Exception) {
        self.except = except.to_string()
    }

    pub fn reset_exception(&mut self) {
        self.except = "".to_string()
    }
}