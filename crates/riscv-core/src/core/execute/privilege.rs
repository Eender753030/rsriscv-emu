use riscv_decoder::instruction::PrivilegeOp::{self, *};

use crate::core::cpu::Cpu;

impl Cpu {
    pub(crate) fn execute_privileged(&mut self, op: PrivilegeOp) {
        let (mode, pc) = match op {
            Mret => self.csrs.trap_mret(),
            Sret => self.csrs.trap_sret(),
        };
        self.pc.directed_addressing(pc);
        self.mode = mode;
    }
}
