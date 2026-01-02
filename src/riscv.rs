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

    fn decode(&self, instruction: u32) -> Result<Instruction, RiscVError> {
        instruction.try_into()
    }

    fn execute(&mut self, op_type: Instruction) -> Result<(), RiscVError> {
        match op_type {
            Instruction::Itype {rd, rs1, imm, funct3} => {
                self.registers.write(
                    rd, 
                    match funct3 {
                        // ADDI
                        0x0 => self.registers.read(rs1)?.wrapping_add_signed(imm),               
                        // SLLI
                        0x1 => self.registers.read(rs1)? << ((imm & 0x1f) as u32),
                        // SLTI
                        0x2 => {
                            match (self.registers.read(rs1)? as i32) < imm {
                                true => 1,
                                false => 0
                            }
                        },
                        // SLTIU
                        0x3 => {
                            match self.registers.read(rs1)? < (imm as u32) {
                                true => 1,
                                false => 0
                            }
                        },
                        // XORI
                        0x4 => self.registers.read(rs1)? ^ (imm as u32),    
                        // SRLI | SRAI
                        0x5 => {
                            match (imm & 0xfe0) >> 5 {
                                // SRLI
                                0x00 => self.registers.read(rs1)? >> ((imm & 0x1f) as u32),
                                // SRAI
                                0x20 => ((self.registers.read(rs1)? as i32) >> (imm & 0x1f)) as u32,
                            
                                not_exist_funct => return Err(RiscVError::NotImplementedFunc(0x13, not_exist_funct as u8))
                            }
                        },
                        // ORI
                        0x6 => self.registers.read(rs1)? | (imm as u32),
                        // ANDI
                        0x7 => self.registers.read(rs1)? & (imm as u32),

                        not_exist_funct => return Err(RiscVError::NotImplementedFunc(0x13, not_exist_funct))
                    }
                )?;
            },

            Instruction::ItypeLoad {rd, rs1, imm, funct3} => {
                self.registers.write(
                    rd,
                    self.data_memory.read(
                        self.registers.read(rs1)?.wrapping_add_signed(imm),
                        match funct3 {
                            0x0 | 0x4 => 1, // LB | LBU
                            0x1 | 0x5 => 2, // LH | LHU
                            0x2 => 4, // LW
                            not_exist_funct => return Err(RiscVError::NotImplementedFunc(0x03, not_exist_funct))
                        },
                        match funct3 {
                            0x0 | 0x1 => true, // LB | LH | LW
                            0x4 | 0x5 | 0x2 => false, // LBU | LHU
                            not_exist_funct => return Err(RiscVError::NotImplementedFunc(0x03, not_exist_funct))
                        },
                    )?
                )?;      
            },

            Instruction::Rtype {rd, rs1, rs2, funct3, funct7} => {
                self.registers.write(
                    rd,
                    match funct3 {
                        // ADD | SUB
                        0x0 => {             
                            match funct7 {
                                // ADD
                                0x00 => self.registers.read(rs1)?.wrapping_add(self.registers.read(rs2)?),
                                // SUB
                                0x20 => self.registers.read(rs1)?.wrapping_sub(self.registers.read(rs2)?),

                                not_exist_funct => return Err(RiscVError::NotImplementedFunc(0x33, not_exist_funct))
                            }                  
                        },
                        // SLL
                        0x1 => self.registers.read(rs1)? << (self.registers.read(rs2)? & 0x1f),
                        // SLT
                        0x2 => {
                            match (self.registers.read(rs1)? as i32) < (self.registers.read(rs2)? as i32) {
                                true => 1,
                                false => 0
                            }
                        },
                        // SLTU
                        0x3 => {
                            match self.registers.read(rs1)? < self.registers.read(rs2)? {
                                true => 1,
                                false => 0
                            }
                        },
                        // XOR
                        0x4 => self.registers.read(rs1)? ^ self.registers.read(rs2)?,
                        // SRL | SRA
                        0x5 => {
                            match funct7 {
                                // SRL
                                0x00 => self.registers.read(rs1)? >> (self.registers.read(rs2)? & 0x1f),
                                // SRA
                                0x20 => (self.registers.read(rs1)? as i32).shr(self.registers.read(rs2)? & 0x1f) as u32,

                                not_exist_funct => return Err(RiscVError::NotImplementedFunc(0x33, not_exist_funct))
                            }   
                        },
                        // OR
                        0x6 => self.registers.read(rs1)? | self.registers.read(rs2)?,
                        // AND
                        0x7 => self.registers.read(rs1)? & self.registers.read(rs2)?,
                        
                        not_exist_funct => return Err(RiscVError::NotImplementedFunc(0x33, not_exist_funct))
                    }
                )?;
            },

            Instruction::Stype {rs1, rs2, imm, funct3} => {
                self.data_memory.write(
                    self.registers.read(rs1)?.wrapping_add_signed(imm), 
                    self.registers.read(rs2)?, 
                    match funct3 {
                        0x0 => 1, // SB
                        0x1 => 2, // SH
                        0x2 => 4, // SW
                        not_exist_funct => return Err(RiscVError::NotImplementedFunc(0x23, not_exist_funct))
                    }
                )?;
            },

            Instruction::Btype {rs1, rs2, imm, funct3} => {
                let rs1_data = self.registers.read(rs1)?;
                let rs2_data = self.registers.read(rs2)?;
                let branch_result = match funct3 {
                    0x0 => rs1_data == rs2_data, // BEQ
                    0x1 => rs1_data != rs2_data, // BNE
                    0x4 => (rs1_data as i32) < (rs2_data as i32), // BLT (Signed)
                    0x5 => (rs1_data as i32) >= (rs2_data as i32), // BGE (Signed)
                    0x6 => rs1_data < rs2_data, // BLTU
                    0x7 => rs1_data >= rs2_data, // BGEU
                    not_exist_funct => return Err(RiscVError::NotImplementedFunc(0x23, not_exist_funct))
                };

                if branch_result {
                    self.pc.related_addressing(imm);
                    return Ok(());
                }
            }

            Instruction::UtypeLUI {rd, imm} => {
                self.registers.write(rd, imm)?;
            },

            Instruction::UtypeAUIPC {rd, imm} => {
                self.registers.write(rd, self.pc.get() + imm)?;
            },

            Instruction::Jtype {rd, imm} => {
                self.registers.write(rd, self.pc.get() + 4)?;
                self.pc.related_addressing(imm);
                return Ok(());
            },

            Instruction::ItypeJump {rd, rs1, imm} => {
                self.registers.write(rd, self.pc.get() + 4)?;
                self.pc.directed_addressing(self.registers.read(rs1)?.wrapping_add_signed(imm) & !1);
                return Ok(());
            },

            Instruction::ItypeSys {imm} => {
                if imm == 0 {
                    let sys_call_id = self.registers.read(17)?;
                    match sys_call_id {
                        64 => { // System write (print)
                            let fd = self.registers.read(10)?;
                            let ptr = self.registers.read(11)? as usize;
                            let len = self.registers.read(12)? as usize;
                            
                            if fd == 1 { // stdout 
                                let slice = self.data_memory.read_batch(ptr, len)?;
                                let s = String::from_utf8_lossy(slice);
                                print!("{}", s);
                            }
                            
                        },

                        93 => {
                            let exit_code = self.registers.read(10)?;

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
            self.registers.dump(),
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