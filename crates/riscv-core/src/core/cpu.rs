use super::{PC, Registers};
// use crate::core::csr::CsrFile;
use crate::device::bus::{Bus, SystemBus};
use crate::engine::*;
use crate::error::RiscVError;
use crate::exception::Exception;
use crate::isa::*;

#[derive(Clone, PartialEq, Default)]
pub struct Cpu {
    reg: Registers,
    pc: PC,
    //   csr: CsrFile,
    bus: SystemBus,
}

impl std::fmt::Debug for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Cpu {{")?;
        writeln!(f, " PC: {:#08X}", self.pc.get())?;
        write!(f, " Registers {{")?;
        self.reg.iter().enumerate().try_for_each(|(id, reg)|
            write!(f, " x{}: {}", id, reg as i32)
        )?;
        writeln!(f, " }}")?;
        write!(f, " {:?}", self.bus)
    }
}

impl Cpu {
    pub fn load(&mut self, code: &[u8]) -> Result<(), RiscVError> {
        if let Err(e) = self.bus.write_bytes(self.pc.get(), code.len(), code) {
            self.trap_handle(e)
        } else {
            Ok(())
        }
    }

    pub fn run(&mut self) -> Result<(), RiscVError> {
        loop {
            if let Err(e) = self.step() {
                break Err(e);
            }
        }
    }

    pub fn step(&mut self) -> Result<(), RiscVError> {
        if let Err(execpt) = self.cycle() {
            self.trap_handle(execpt)?;
        }
        Ok(())
    }

    pub fn cycle(&mut self) -> Result<(), Exception> {
        let instruction = self.fetch()?;
        
        let type_data = self.decode(instruction)?;

        self.execute(type_data)?;
        Ok(())
    }

    fn fetch(&mut self) -> Result<u32, Exception> {
        self.bus.read_u32(self.pc.get())
    }

    fn decode(&self, bytes: u32) -> Result<Instruction, Exception> {
        decoder::decode(bytes)
    }

    fn execute(&mut self, ins: Instruction) -> Result<(), Exception> {
        match ins {
            Instruction::Base(op, data) => {
                if self.execute_rv32i(op, data)? {
                    return Ok(());
                }
            },
            // Instruction::System(op, data) => {},
        }

        self.pc.step();
        Ok(())
    }

    fn execute_rv32i(&mut self, op: Rv32iOp, data: InstructionData) -> Result<bool, Exception> {
        let rs1_data = self.reg[data.rs1];
        let rs2_data = self.reg[data.rs2];
        let mut branch = false;
        let mut jump = false;

        match op {
            Rv32iOp::Addi => self.reg.write(data.rd, ALU::add_signed(rs1_data, data.imm)),
            Rv32iOp::Slli => self.reg.write(data.rd, ALU::shl_logic(rs1_data, data.imm as u32)),
            Rv32iOp::Slti => self.reg.write(data.rd, ALU::set_less_than(rs1_data as i32, data.imm)),
            Rv32iOp::Sltiu => self.reg.write(data.rd, ALU::set_less_than_unsigned(rs1_data, data.imm as u32)),
            Rv32iOp::Xori => self.reg.write(data.rd, ALU::xor(rs1_data, data.imm as u32)),
            Rv32iOp::Srli => self.reg.write(data.rd, ALU::shr_logic(rs1_data, data.imm as u32)),
            Rv32iOp::Srai => self.reg.write(data.rd, ALU::shr_ar(rs1_data as i32, data.imm as u32)),
            Rv32iOp::Ori => self.reg.write(data.rd, ALU::or(rs1_data, data.imm as u32)),
            Rv32iOp::Andi => self.reg.write(data.rd, ALU::and(rs1_data, data.imm as u32)),

            Rv32iOp::Add => self.reg.write(data.rd, ALU::add(rs1_data, rs2_data)),
            Rv32iOp::Sub => self.reg.write(data.rd, ALU::sub(rs1_data, rs2_data)),
            Rv32iOp::Sll => self.reg.write(data.rd, ALU::shl_logic(rs1_data, rs2_data)),
            Rv32iOp::Slt => self.reg.write(data.rd, ALU::set_less_than(rs1_data as i32, rs2_data as i32)),
            Rv32iOp::Sltu => self.reg.write(data.rd, ALU::set_less_than_unsigned(rs1_data, rs2_data)),
            Rv32iOp::Xor => self.reg.write(data.rd, ALU::xor(rs1_data, rs2_data)),
            Rv32iOp::Srl => self.reg.write(data.rd, ALU::shr_logic(rs1_data, rs2_data)),
            Rv32iOp::Sra => self.reg.write(data.rd, ALU::shr_ar(rs1_data as i32, rs2_data)),
            Rv32iOp::Or => self.reg.write(data.rd, ALU::or(rs1_data, rs2_data)),
            Rv32iOp::And => self.reg.write(data.rd, ALU::and(rs1_data, rs2_data)),

            Rv32iOp::Lb => self.reg.write(data.rd, LSU::load(&mut self.bus, rs1_data, data.imm, 1)?),
            Rv32iOp::Lh => self.reg.write(data.rd, LSU::load(&mut self.bus, rs1_data, data.imm, 2)?),
            Rv32iOp::Lw => self.reg.write(data.rd, LSU::load(&mut self.bus, rs1_data, data.imm, 4)?),
            Rv32iOp::Lbu => self.reg.write(data.rd, LSU::load_signed(&mut self.bus, rs1_data, data.imm, 1)?),
            Rv32iOp::Lhu => self.reg.write(data.rd, LSU::load_signed(&mut self.bus, rs1_data, data.imm, 2)?),

            Rv32iOp::Sb => LSU::store(&mut self.bus, rs1_data, rs2_data, data.imm, 1)?,
            Rv32iOp::Sh => LSU::store(&mut self.bus, rs1_data, rs2_data, data.imm, 2)?,
            Rv32iOp::Sw => LSU::store(&mut self.bus, rs1_data, rs2_data, data.imm, 4)?,

            Rv32iOp::Beq => branch = Branch::equal(rs1_data, rs2_data),
            Rv32iOp::Bne => branch = Branch::not_equal(rs1_data, rs2_data),
            Rv32iOp::Blt => branch = Branch::less(rs1_data as i32, rs2_data as i32),
            Rv32iOp::Bge => branch = Branch::greater_eqaul(rs1_data as i32, rs2_data as i32),
            Rv32iOp::Bltu => branch = Branch::less_unsigned(rs1_data, rs2_data),
            Rv32iOp::Bgeu => branch = Branch::greater_eqaul_unsigned(rs1_data, rs2_data),

            Rv32iOp::Jal => {
                self.reg.write(data.rd, self.pc.get() + 4);
                self.pc.related_addressing(data.imm);
                jump = true;
            },
            Rv32iOp::Jalr => {
                self.reg.write(data.rd, self.pc.get() + 4);
                self.pc.directed_addressing(rs1_data.wrapping_add_signed(data.imm));
                jump = true;
            },

            Rv32iOp::Lui => self.reg.write(data.rd, data.imm as u32),
            Rv32iOp::Auipc => self.reg.write(data.rd, self.pc.get() + data.imm as u32),

            Rv32iOp::Fence => {},

            Rv32iOp::Ecall => {},
            Rv32iOp::Ebreak => {},
        }

        if branch {
            self.pc.related_addressing(data.imm);
        }

        Ok(branch | jump)
    }

    fn trap_handle(&mut self, except: Exception) -> Result<(), RiscVError> {
        let _ = except;

        Err(RiscVError::EndOfInstruction)
    }

    pub fn reset(&mut self) {
        self.bus.reset_ram();
        self.pc.reset();
        self.reg.reset();
    }

    pub fn dump_data(&self) -> (Vec<i32>, Vec<[u8; 4]>, u32) {
        (
            self.reg.dump(),
            vec![[0; 4]; 10], // Give 0 to let compile pass
            self.pc.get(),
        )
    }
}
