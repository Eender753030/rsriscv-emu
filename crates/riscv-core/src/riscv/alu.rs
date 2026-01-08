use crate::error::RiscVError;
use super::instruction::OpCode;
pub struct ALU;

impl ALU {
    pub fn itype (data:u32, imm: i32, funct3: u8) -> Result<u32, RiscVError> {
        Ok(match funct3 {
            // ADDI
            0x0 => data.wrapping_add_signed(imm),               
            // SLLI
            0x1 => data << ((imm & 0x1f) as u32),
            // SLTI
            0x2 => {
                match (data as i32) < imm {
                    true => 1,
                    false => 0
                }
            },
            // SLTIU
            0x3 => {
                match data < (imm as u32) {
                    true => 1,
                    false => 0
                }
            },
            // XORI
            0x4 => data ^ (imm as u32),    
            // SRLI | SRAI
            0x5 => {
                match (imm & 0xfe0) >> 5 {
                    // SRLI
                    0x00 => data >> ((imm & 0x1f) as u32),
                    // SRAI
                    0x20 => ((data as i32) >> (imm & 0x1f)) as u32,
                
                    not_exist_funct => return Err(RiscVError::NotImplementedFunc(OpCode::ITYPE, not_exist_funct as u8))
                }
            },
            // ORI
            0x6 => data | (imm as u32),
            // ANDI
            0x7 => data & (imm as u32),

            not_exist_funct => return Err(RiscVError::NotImplementedFunc(OpCode::ITYPE, not_exist_funct))
        })
    }

    pub fn rtype(data1: u32, data2: u32, funct3: u8, funct7: u8) -> Result<u32, RiscVError> {
        Ok(match funct3 {
            // ADD | SUB
            0x0 => {             
                match funct7 {
                    // ADD
                    0x00 => data1.wrapping_add(data2),
                    // SUB
                    0x20 => data1.wrapping_sub(data2),

                    not_exist_funct => return Err(RiscVError::NotImplementedFunc(OpCode::RTYPE, not_exist_funct))
                }                  
            },
            // SLL
            0x1 => data1 << (data2 & 0x1f),
            // SLT
            0x2 => {
                match (data1 as i32) < (data2 as i32) {
                    true => 1,
                    false => 0
                }
            },
            // SLTU
            0x3 => {
                match data1 < data2 {
                    true => 1,
                    false => 0
                }
            },
            // XOR
            0x4 => data1 ^ data2,
            // SRL | SRA
            0x5 => {
                match funct7 {
                    // SRL
                    0x00 => data1 >> (data2 & 0x1f),
                    // SRA
                    0x20 => ((data1 as i32) << (data2 & 0x1f)) as u32,

                    not_exist_funct => return Err(RiscVError::NotImplementedFunc(OpCode::RTYPE, not_exist_funct))
                }   
            },
            // OR
            0x6 => data1 | data2,
            // AND
            0x7 => data1 & data2,
            
            not_exist_funct => return Err(RiscVError::NotImplementedFunc(OpCode::RTYPE, not_exist_funct))
        })   
    }

    pub fn btype(data1: u32, data2: u32, funct3: u8) -> Result<bool, RiscVError> {
        Ok(match funct3 {
            0x0 => data1 == data2, // BEQ
            0x1 => data1 != data2, // BNE
            0x4 => (data1 as i32) < (data2 as i32), // BLT (Signed)
            0x5 => (data1 as i32) >= (data2 as i32), // BGE (Signed)
            0x6 => data1 < data2, // BLTU
            0x7 => data1 >= data2, // BGEU
            not_exist_funct => return Err(RiscVError::NotImplementedFunc(OpCode::BTYPE, not_exist_funct))
        })
    }
}