mod register;
mod pc;
mod memory;
mod opcode;
pub mod loader;

use register::Registers;
use pc::PC;
use memory::Memory;
use crate::{riscv::opcode::{TypeKind, Types}, utils::exception::RiscVError};

pub struct RiscV {
    registers: Registers,
    pc: PC,
    ins_memory: Memory,
    data_memory: Memory,
}

impl RiscV {
    pub fn new() -> Self {
        RiscV {
            registers: Registers::new(),
              pc: PC::new(),
            ins_memory: Memory::new(128),
            data_memory: Memory::new(512),
        }
    }
    
    pub fn cycle(&mut self, code: &[u8]) -> Result<(), RiscVError>{
        self.ins_memory.load(0, code)?;

        loop {
            let instruction = self.fetch()?;
            if instruction == 0 {
                break Ok(());
            }
            
            let type_data = self.decode(instruction)?;
            
            self.execute(type_data)?;
        }
    }

    fn fetch(&self) -> Result<u32, RiscVError> {
        self.ins_memory.fetch(self.pc.get())
    }

    fn decode(&self, instruction: u32) -> Result<Types, RiscVError>{
        match instruction & 0x7f {
            0x13 => {  
                    Ok(opcode::Types::parse(TypeKind::IType, instruction, ((instruction & 0x7000) >> 12) as u16))           
            },
            
            not_exist_opcode => {
                Err(RiscVError::NotImplementedOpCode(not_exist_opcode))
            }
        }
    }

    fn execute(&mut self, op_type: Types) -> Result<(), RiscVError> {
        match op_type {
            Types::IType {imm, rs1, rd, func} => {
                match func {
                    // ADDI
                    0x0 => { 
                        self.registers.write(rd, self.registers.read(rs1)?.wrapping_add(imm as u32))?; 
                    },

                    not_exist_func => {
                        return Err(RiscVError::NotImplementedFunc(0x13, not_exist_func))
                    }
                }
            }
        }

        self.pc.step();
        Ok(())
    }

    pub fn print(&self) {
        println!("{:?}\n{:?}", self.registers, self.pc);
    }
}