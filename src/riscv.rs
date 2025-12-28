mod register;
mod pc;
mod memory;
mod opcode;

use register::Registers;
use pc::PC;
use memory::Memory;
use crate::{riscv::opcode::{TypeKind, Types}, utils::exception::RiscVError};

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
        self.memory.read(self.pc.get())
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

    pub fn load_code(&mut self, code: u32) -> Result<(), RiscVError>{ 
        self.memory.write(self.pc.get(), code)
    }

    pub fn print(&self) {
        println!("{:?}\n{:?}", self.registers, self.pc);
    }
}