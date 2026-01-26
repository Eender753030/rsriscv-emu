use riscv_decoder::instruction::InstructionData;
use riscv_decoder::instruction::ZicsrOp;

use crate::exception::Exception;
use crate::core::cpu::Cpu;

impl Cpu {
    pub(crate) fn execute_zicsr(&mut self, op: ZicsrOp, data: InstructionData) -> Result<(), Exception> {
        let addr = (data.imm & 0xfff) as u16;
        let (val, check_val) = if op.is_imm() {
            (data.rs1 as u32, data.rs1)
        } else {
            (self.regs[data.rs1], data.rs1)
        };

        if op.is_rw() {
            if data.rd != 0 {
                let csr_data = self.csrs.read(addr, self.mode)?;
                self.csrs.write(addr, val, self.mode)?;
                self.regs.write(data.rd, csr_data);
            } else {
                self.csrs.write(addr, val, self.mode)?;
            }
        } else {
            let csr_data = self.csrs.read(addr, self.mode)?;
            if check_val != 0 {
                let write_val = if op.is_rs() {
                    val | csr_data
                } else { // is_rc
                    (!val) & csr_data
                };
                self.csrs.write(addr, write_val, self.mode)?;
            }
            self.regs.write(data.rd, csr_data);
        }

        Ok(())
    }
}
