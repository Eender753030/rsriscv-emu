mod register;
mod pc;
mod memory;
mod opcode;

use register::Registers;
use pc::PC;
use memory::Memory;
use crate::{riscv::opcode::{Action, TypeKind, Types}, utils::exception::RiscVError};

pub struct RiscV {
    registers: Registers,
    pc: PC,
    memory: Memory,
}

impl RiscV {
    pub fn new() -> Self {
        RiscV{registers: Registers::new(),
              pc: PC::new(),
              memory: Memory::new(1024)}
    }
    
    pub fn cycle(&mut self) -> Result<(), RiscVError>{
        let instruction = self.fetch()?;

        let type_data = self.decode(instruction)?;
        
        self.execute(type_data)?;
        Ok(())
    }

    fn fetch(&self) -> Result<u32, RiscVError> {
        self.memory.read(self.pc.get() as usize)
    }

    fn decode(&self, instruction: u32) -> Result<Types, RiscVError>{
        match instruction & 0x7f {
            0x13 => {
                if instruction & 0x7000 == 0 {
                   return Ok(opcode::Types::parse(TypeKind::IType, instruction, opcode::Action::ADDI));
                } else {
                    Err(RiscVError::InvalidRegister(0))
                }
            }

            _ => {
                Err(RiscVError::InvalidRegister(0))
            }
        }
    }

    fn execute(&mut self, op_type: Types) -> Result<(), RiscVError> {
        match op_type {
            Types::IType {imm, rs1, rd, action} => {
                match action {
                    Action::ADDI => {
                        self.registers.write(rd, self.registers.read(rs1)? + imm as u32)?; 
                    }
                }
            }
        }

        self.pc.step();
        Ok(())
    }

    pub fn load_code(&mut self, code: u32) -> Result<(), RiscVError>{
        self.memory.write(0, code)
    }

    pub fn print_registers(&self) {
        self.registers.print();
    }
}