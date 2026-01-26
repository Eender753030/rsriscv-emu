use riscv_decoder::instruction::InstructionData;
use riscv_decoder::instruction::MOp::{self, *};

use crate::engine::Alu;
use crate::core::cpu::Cpu;

impl Cpu {
    pub(crate) fn execute_m(&mut self, op: MOp, data: InstructionData) {
        let rs1_data = self.regs[data.rs1];
        let rs2_data = self.regs[data.rs2];
        
        self.regs.write(data.rd, 
            match op {
                Mul    => Alu::mul(rs1_data, rs2_data),
                Mulh   => Alu::mulh(rs1_data, rs2_data),
                Mulhu  => Alu::mulh_unsigned(rs1_data, rs2_data),
                Mulhsu => Alu::mulh_signed_unsigned(rs1_data, rs2_data),
                Div    => Alu::div(rs1_data, rs2_data),
                Divu   => Alu::div_unsigned(rs1_data, rs2_data),
                Rem    => Alu::rem(rs1_data, rs2_data),
                Remu   => Alu::rem_unsigned(rs1_data, rs2_data),
            }
        )
    }   
}