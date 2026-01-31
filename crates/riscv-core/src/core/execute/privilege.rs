#![allow(unused)]

use riscv_decoder::instruction::InstructionData;
use riscv_decoder::instruction::PrivilegeOp::{self, *};

use crate::{Exception, Result};
use crate::core::PrivilegeMode;
use crate::core::cpu::Cpu;

impl Cpu {
    pub(crate) fn execute_privileged(&mut self, op: PrivilegeOp, data: InstructionData) -> Result<bool> {
        let (mode, pc) = match op {
            Mret           => self.csrs.trap_mret(),
            #[cfg(feature = "s")]
            Sret           => self.csrs.trap_sret(self.mode)?,
            #[cfg(feature = "s")]
            SfenceVma(raw) => {
                if self.mode == PrivilegeMode::User {
                    return Err(Exception::IllegalInstruction(raw));
                }  
                if self.mode == PrivilegeMode::Supervisor && self.csrs.check_tvm() {
                    return Err(Exception::IllegalInstruction(raw));
                }
                let rs1_data = self.regs[data.rs1];
                let rs2_data = self.regs[data.rs2];
                self.mmu.flush_tlb(rs1_data, rs2_data);
                return Ok(false);
            }
        };
        self.pc.directed_addressing(pc);
        self.mode = mode;
        Ok(true)
    }
}
