mod register;
mod pc;
mod memory;
mod instruction;
pub mod loader;

use std::ops::Shr;

use register::Registers;
use pc::PC;
use memory::Memory;
use crate::{riscv::instruction::Instruction, utils::exception::RiscVError};

pub trait Reset {
    fn reset(&mut self);   
}

pub trait Dump<T> {
    fn dump(&self) -> Vec<T>;   
}

pub struct RiscV {
    registers: Registers,
    pc: PC,
    ins_memory: Memory,
    data_memory: Memory,
}

impl RiscV {
    pub fn new(memory_size: usize) -> Self {
        RiscV { 
            ins_memory: Memory::new(memory_size / 2), 
            data_memory: Memory::new(memory_size),
            ..Default::default()
        }
    }
    
    pub fn load(&mut self, code: &[u8]) -> Result<(), RiscVError> {
        self.ins_memory.load(self.pc.get() as usize, code)
    }

    pub fn cycle(&mut self) -> Result<(), RiscVError>{
        loop {
            if let Err(e) = self.step() {
                break match e {
                    RiscVError::EndOfInstruction | RiscVError::SystemExit(_) => Ok(()),
                    _ => Err(e)
                }
            }
        }
    }

    pub fn step(&mut self) -> Result<(), RiscVError>{   
        let instruction = self.fetch()?;
        if instruction == 0 {
            return Err(RiscVError::EndOfInstruction);
        }
        
        let type_data = self.decode(instruction)?;
        
        self.execute(type_data)?;
        Ok(())
    }

    fn fetch(&self) -> Result<u32, RiscVError> {
        self.ins_memory.fetch(self.pc.get())
    }

    fn decode(&self, bytes: u32) -> Result<Instruction, RiscVError> {
        bytes.try_into()
    }

    fn execute(&mut self, ins: Instruction) -> Result<(), RiscVError> {
        match ins {
            Instruction::Itype {rd, rs1, imm, funct3} => {
                let data = self.reg.read(rs1)?;
                self.reg.write(rd, ALU::itype(data, imm, funct3)?)?;
            },

            Instruction::ItypeLoad {rd, rs1, imm, funct3} => {
                let addr = self.reg.read(rs1)?.wrapping_add_signed(imm);
                let bytes_amount = match funct3 {
                            0x0 | 0x4 => 1, // LB | LBU
                            0x1 | 0x5 => 2, // LH | LHU
                            0x2 => 4, // LW
                    not_exist_funct => return Err(RiscVError::NotImplementedFunc(OpCode::ITYPE_LOAD, not_exist_funct))
                };
                let is_signed = match funct3 {
                            0x0 | 0x1 => true, // LB | LH | LW
                            0x4 | 0x5 | 0x2 => false, // LBU | LHU
                    not_exist_funct => return Err(RiscVError::NotImplementedFunc(OpCode::ITYPE_LOAD, not_exist_funct))
                };
                let data = self.data_memory.read(addr, bytes_amount, is_signed)?;
                self.reg.write(rd, data)?;      
            },

            Instruction::Rtype {rd, rs1, rs2, funct3, funct7} => {
                let data1 = self.reg.read(rs1)?;
                let data2 = self.reg.read(rs2)?;
                self.reg.write(rd, ALU::rtype(data1, data2, funct3, funct7)?)?;
            },

            Instruction::Stype {rs1, rs2, imm, funct3} => {
                let source = self.reg.read(rs2)?;
                let addr = self.reg.read(rs1)?.wrapping_add_signed(imm);
                let bytes_amount = match funct3 {
                        0x0 => 1, // SB
                        0x1 => 2, // SH
                        0x2 => 4, // SW
                    not_exist_funct => return Err(RiscVError::NotImplementedFunc(OpCode::STYPE, not_exist_funct))
                };
                self.data_memory.write(addr, source, bytes_amount)?;
            },

            Instruction::Btype {rs1, rs2, imm, funct3} => {
                let data1 = self.reg.read(rs1)?;
                let data2 = self.reg.read(rs2)?;
                let branch_result = ALU::btype(data1, data2, funct3)?;

                if branch_result {
                    self.pc.related_addressing(imm);
                    return Ok(());
                }
            }

            Instruction::UtypeLUI {rd, imm} => {
                self.reg.write(rd, imm)?;
            },

            Instruction::UtypeAUIPC {rd, imm} => {
                self.reg.write(rd, self.pc.get() + imm)?;
            },

            Instruction::Jtype {rd, imm} => {
                self.reg.write(rd, self.pc.get() + 4)?;
                self.pc.related_addressing(imm);
                return Ok(());
            },

            Instruction::ItypeJump {rd, rs1, imm} => {
                self.reg.write(rd, self.pc.get() + 4)?;
                self.pc.directed_addressing(self.reg.read(rs1)?.wrapping_add_signed(imm) & !1);
                return Ok(());
            },

            Instruction::ItypeSys {imm} => {
                if imm == 0 {
                    let sys_call_id = self.reg.read(17)?;
                    match sys_call_id {
                        64 => { // System write (print)
                            let fd = self.reg.read(10)?;
                            let ptr = self.reg.read(11)? as usize;
                            let len = self.reg.read(12)? as usize;
                            
                            if fd == 1 { // stdout 
                                let slice = self.data_memory.read_batch(ptr, len)?;
                                let s = String::from_utf8_lossy(slice);
                                print!("{}", s);
                            }
                            
                        },

                        93 => {
                            let exit_code = self.reg.read(10)?;

                            return Err(RiscVError::SystemExit(exit_code));
                        },

                        _ => {
                            return Err(RiscVError::NotImplementedSysCall(sys_call_id));
                        }
                    }
                }
            }
        }

        self.pc.step();
        Ok(())
    }

    pub fn dump_data(&self) -> (Vec<i32>, Vec<[u8; 4]>, u32) {
        (
            self.reg.dump(),
            self.data_memory.dump(),
            self.pc.get()
        )
    }

    pub fn dump_ins(&mut self) -> Result<Vec<String>, RiscVError> {
        let mut ins_list = vec![];
        loop { 
            match self.decode(self.fetch()?) {
                Ok(decoded) => {
                    ins_list.push(decoded.to_string());
                    self.pc.step();            
                }
                Err(RiscVError::EndOfInstruction) => {
                    self.pc.reset();
                    break Ok(ins_list);
                },
                Err(e) => break Err(e)
            }
        }
    }
}

impl Default for RiscV {
    fn default() -> Self {
        RiscV {
            registers: Registers::default(),
            pc: PC::default(),
            ins_memory: Memory::new(512),
            data_memory: Memory::default(),
        }
    }
}

impl Reset for RiscV {
    fn reset(&mut self) {
        self.registers.reset();
        self.data_memory.reset();
        self.pc.reset();
    }
}

impl std::fmt::Debug for RiscV {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Registers {{ {:?} }}\nPC: {}\nMemory: {:?}", self.registers.dump(), self.pc.get(), self.data_memory.dump())
    }
}