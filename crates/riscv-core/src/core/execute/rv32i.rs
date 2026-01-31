use riscv_decoder::instruction::InstructionData;
use riscv_decoder::instruction::Rv32iOp::{self, *};

use crate::{Exception, Result};
use crate::engine::{Alu, Branch, Lsu};
use crate::core::cpu::Cpu;

impl Cpu {
    pub(crate) fn execute_rv32i(&mut self, op: Rv32iOp, data: InstructionData) -> Result<bool> {
        let rs1_data = self.regs[data.rs1];
        let rs2_data = self.regs[data.rs2];

        if let Some(res) = Self::alu_imm(op, rs1_data, data.imm, self.pc.get()) {
            self.regs.write(data.rd, res);
            return Ok(false);
        } 
        
        if let Some(res) = Self::alu_reg(op, rs1_data, rs2_data) {
            self.regs.write(data.rd, res);
            return Ok(false);
        } 

        if let Some(res) = self.lsu_load(op, rs1_data, data.imm) {
            self.regs.write(data.rd, res?);
            return Ok(false);
        }

        if let Some(res) = self.lsu_store(op, rs1_data, rs2_data, data.imm) {
            res?;
            return Ok(false);
        }

        if let Some(res) = Self::branch(op, rs1_data, rs2_data) {
            if res {
                self.pc.related_addressing(data.imm)
            }
            return Ok(res);
        }

        if let Some(res) = Self::jump(op) {
            #[cfg(feature = "c")]
            let next_ins_addr = if self.is_compress {
                2
            } else {
                4
            };
            #[cfg(not(feature = "c"))]
            let next_ins_addr = 4;
            
            self.regs.write(data.rd, self.pc.get() + next_ins_addr);
            match res {
                true  => self.pc.directed_addressing(rs1_data.wrapping_add_signed(data.imm)),
                false => self.pc.related_addressing(data.imm),
            }
            return Ok(true);
        }

        if let Some(res) = self.system(op) {
            res?;
        }
        Ok(false)
    }

    fn alu_imm(op: Rv32iOp, data: u32, imm: i32, pc: u32) -> Option<u32> {
        Some(match op {
            Addi  => Alu::add_signed(data, imm),
            Slli  => Alu::shl_logic(data, imm as u32),
            Slti  => Alu::set_less_than(data as i32, imm),
            Sltiu => Alu::set_less_than_unsigned(data, imm as u32),
            Xori  => Alu::xor(data, imm as u32),
            Srli  => Alu::shr_logic(data, imm as u32),
            Srai  => Alu::shr_ar(data as i32, imm as u32),
            Ori   => Alu::or(data, imm as u32),
            Andi  => Alu::and(data, imm as u32),

            Lui   => imm as u32,
            Auipc => Alu::add(pc, imm as u32),
            _     => return None,
        })
    }

    fn alu_reg(op: Rv32iOp, data1: u32, data2: u32) -> Option<u32> {
        Some(match op {
            Add  => Alu::add(data1, data2),
            Sub  => Alu::sub(data1, data2),
            Sll  => Alu::shl_logic(data1, data2),
            Slt  => Alu::set_less_than(data1 as i32, data2 as i32),
            Sltu => Alu::set_less_than_unsigned(data1, data2),
            Xor  => Alu::xor(data1, data2),
            Srl  => Alu::shr_logic(data1, data2),
            Sra  => Alu::shr_ar(data1 as i32, data2),
            Or   => Alu::or(data1, data2),
            And  => Alu::and(data1, data2),
            _    => return None,
        })
    }

    fn lsu_load(&mut self, op: Rv32iOp, src: u32, offset: i32) -> Option<Result<u32>> {
        let (is_signed, byte_num) = match op {
            Lb  => (true, 1),
            Lh  => (true, 2),
            Lw  => (false, 4),
            Lbu => (false, 1),
            Lhu => (false, 2),
            _   => return None,
        };

        let mut lsu = Lsu::new(
            &mut self.bus, 
            #[cfg(feature = "s")] &mut self.mmu, 
            #[cfg(feature = "zicsr")] &self.csrs, 
            #[cfg(feature = "zicsr")] self.mode
        );
 
        Some(if is_signed {
                lsu.load_signed(src, offset, byte_num)
            } else {
                lsu.load(src, offset, byte_num)
            }
        )
    }

    fn lsu_store(&mut self, op: Rv32iOp, des: u32, src: u32, offset: i32) -> Option<Result<()>> {
        let byte_num = match op {
            Sb => 1,
            Sh => 2,
            Sw => 4,
            _  => return None,
        };
        let mut lsu = Lsu::new(
            &mut self.bus,
            #[cfg(feature = "s")] &mut self.mmu, 
            #[cfg(feature = "zicsr")] &self.csrs, 
            #[cfg(feature = "zicsr")] self.mode
        );

        Some(lsu.store(des, src, offset, byte_num, 
            #[cfg(feature = "a")] &mut self.reservation))
    }

    fn branch(op: Rv32iOp, data1: u32, data2: u32) -> Option<bool> {
        Some(match op {
            Beq  => Branch::equal(data1, data2),
            Bne  => Branch::not_equal(data1, data2),
            Blt  => Branch::less(data1 as i32, data2 as i32),
            Bge  => Branch::greater_eqaul(data1 as i32, data2 as i32),
            Bltu => Branch::less_unsigned(data1, data2),
            Bgeu => Branch::greater_eqaul_unsigned(data1, data2),
            _    => return None,
        })
    }

    fn jump(op: Rv32iOp) -> Option<bool> {
        Some(match op {
            Jal  => false,
            Jalr => true,
            _    => return None,
        })
    }

    fn system(&self, op: Rv32iOp) -> Option<Result<()>> {
        Some(match op {
            Fence  => Ok(()),
            Ecall  => {
                #[cfg(not(feature = "zicsr"))]
                return Some(Err(Exception::Ebreak));
                #[cfg(feature = "zicsr")]
                Err(self.mode.call_exception())
            },
            Ebreak => {
                #[cfg(not(feature = "zicsr"))]
                return Some(Err(Exception::Ecall));
                #[cfg(feature = "zicsr")]
                Err(Exception::Breakpoint)
            }
            _ => unreachable!("Last of Rv32i op"),
        })
    }
}