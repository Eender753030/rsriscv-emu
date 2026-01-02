use crate::utils::exception::RiscVError;

use std::fmt::Display;

pub struct OpCode;

impl OpCode {
    pub const ITYPE: u32 = 0x13;
    pub const ITYPE_LOAD: u32 = 0x03;
    pub const ITYPE_JUMP: u32 = 0x67;
    pub const ITYPE_SYS: u32 = 0x73;
    pub const RTYPE: u32 = 0x33;
    pub const STYPE: u32 = 0x23;
    pub const BTYPE: u32 = 0x63;
    pub const JTYPE: u32 = 0x6f;
    pub const UTYPE_AUIPC: u32 = 0x17;
    pub const UTYPE_LUI: u32 = 0x37;
}

#[derive(Debug)]
pub enum Instruction {
    Itype {rd: usize, rs1: usize, imm: i32, funct3: u8},
    ItypeLoad {rd: usize, rs1: usize, imm: i32, funct3: u8},
    ItypeJump {rd: usize, rs1: usize, imm: i32},
    ItypeSys {imm: i32},
    Rtype {rd: usize, rs1: usize, rs2: usize, funct3: u8, funct7: u8},
    Stype {rs1: usize, rs2: usize, imm: i32, funct3: u8},
    Btype {rs1: usize, rs2: usize, imm: i32, funct3: u8},
    UtypeLUI {rd: usize, imm: u32},
    UtypeAUIPC {rd: usize, imm: u32},
    Jtype {rd: usize, imm: i32},
}

impl TryFrom<u32> for Instruction {
    type Error = RiscVError;
    fn try_from(ins: u32) -> Result<Self, Self::Error> {
        let rd = ((ins >> 7) & 0x1f) as usize;
        let rs1 = ((ins >> 15) & 0x1f) as usize;
        let rs2 = ((ins >> 20) & 0x1f) as usize;
        let funct3 = ((ins >> 12) & 0x7) as u8;
        let funct7 = ((ins >> 25) & 0x7f) as u8;

        Ok(match ins & 0x7f {
            OpCode::ITYPE => {
                let imm = ((ins & 0xfff00000) as i32) >> 20;
                Instruction::Itype {rd, rs1, imm, funct3}
            },
            OpCode::ITYPE_LOAD => {
                let imm = ((ins & 0xfff00000) as i32) >> 20;
                Instruction::ItypeLoad {rd, rs1, imm, funct3}
            },
            OpCode::ITYPE_JUMP => {
                let imm = ((ins & 0xfff00000) as i32) >> 20;
                Instruction::ItypeJump {rd, rs1, imm}
            },
            OpCode::ITYPE_SYS => {
                let imm = ((ins & 0xfff00000) as i32) >> 20;
                Instruction::ItypeSys {imm}
            },
            OpCode::RTYPE => {
                Instruction::Rtype {rd, rs1, rs2, funct3, funct7}
            },
            OpCode::STYPE => {
                let imm = (((ins & 0xfe000000) as i32) >> 20) | (((ins >> 7) & 0x1f) as i32);
                Instruction::Stype {rs1, rs2, imm, funct3}
            },
            OpCode::BTYPE => {
                let imm = (((ins & 0x80000000) as i32) >> 19) | ((((ins & 0x80) << 4) | ((ins & 0x7e000000) >> 20) | ((ins & 0xf00) >> 7)) as i32);
                Instruction::Btype {rs1, rs2, imm, funct3}
            },
            OpCode::JTYPE => {
                let imm = (((ins & 0x80000000) as i32) >> 11) | (((ins & 0xff000) | ((ins & 0x100000) >> 9) | ((ins & 0x7fe00000) >> 20)) as i32);
                Instruction::Jtype {rd, imm}
            },
            OpCode::UTYPE_LUI => {
                let imm = ins & 0xfffff000;
                Instruction::UtypeLUI {rd, imm}
            },
            OpCode::UTYPE_AUIPC => {
                let imm = ins & 0xfffff000;
                Instruction::UtypeAUIPC {rd, imm}  
            },  
            0x0 => {
                return Err(RiscVError::EndOfInstruction);
            }
            not_exist_opcode => {
                return Err(RiscVError::NotImplementedOpCode(not_exist_opcode))
            }
        })
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Itype {rd, rs1, imm, funct3} => {  
                write!(
                    f, "{} x{}, x{}, {}",
                    match funct3 {
                        0x0 => "addi   ",               
                        0x1 => "slli   ",
                        0x2 => "slti   ",
                        0x3 => "sltiu  ",
                        0x4 => "xori   ",  
                        0x5 => {
                            match (imm & 0xfe0) >> 5 {
                                0x00 => "srli   ",
                                0x20 => "srai   ",
                                _ => " "
                            }
                        },
                        0x6 => "ori    ",
                        0x7 => "andi   ",
                        _ => " "
                    },
                    rd, rs1, imm
                )
            },
            Instruction::ItypeLoad {rd, rs1, imm, funct3} => {
                write!(
                    f, "{} x{}, {}(x{})",
                    match funct3 {
                        0x0 => "lb     ",
                        0x1 => "lh     ",
                        0x2 => "lw     ",
                        0x4 => "lbu    ",
                        0x5 => "lhu    ",
                        _ => " "
                    },
                    rd, imm, rs1
                )
            },
            Instruction::ItypeJump {rd, rs1, imm} => {
                write!(
                    f, "jalr    x{}, {}(x{})    # Go {} steps: {}",
                    rd, imm, rs1,   
                    match imm.is_positive() {
                        true => "forward",
                        false => "backward"
                    },
                    (imm >> 1).abs()
                )
            },
            Instruction::ItypeSys {imm} => {
                write!(
                    f, "{}",
                    match imm {
                        0 => "ecall",
                        1 => "ebreak",
                        _ => " "
                    }
                )
            },
            Instruction::Rtype {rd, rs1, rs2, funct3, funct7} => {
                write!(
                    f, "{}  x{}, x{}, x{}",
                    match funct3 {
                        0x0 => {             
                            match funct7 {
                                0x00 => "add    ",
                                0x20 => "sub    ",
                                _ => " "
                            }                  
                        },
                        0x1 => "sll    ",
                        0x2 => "slt    ",
                        0x3 => "sltu   ",
                        0x4 => "xor    ",
                        0x5 => {
                            match funct7 {
                                0x00 => "srl    ",
                                0x20 => "sra    ",
                                _ => " "
                            }   
                        },
                        0x6 => "or     ",
                        0x7 => "and    ",
                        _ => " "
                    },
                    rd, rs1, rs2
                )
            },
            Instruction::Stype {rs1, rs2, imm, funct3} => {
                write!(
                    f, "{} x{}, {}(x{})",
                    match funct3 {
                        0x0 => "sb     ", 
                        0x1 => "sh     ", 
                        0x2 => "sw     ", 
                        _ => " "
                    },
                    rs2, imm, rs1)
            },
            Instruction::Btype {rs1, rs2, imm, funct3} => { 
                write!(
                    f, "{} x{}, x{}, {}    # {} steps: {}",
                    match funct3 {
                        0x0 => "beq    ",
                        0x1 => "bne    ",
                        0x4 => "blt    ",
                        0x5 => "bge    ",
                        0x6 => "bltu   ",
                        0x7 => "bgeu   ",
                        _ => " "
                    },
                    rs1, rs2, imm << 1, 
                    match imm.is_positive() {
                        true => "forward",
                        false => "backward"
                    },
                    (imm >> 1).abs()
                )
            },
            Instruction::Jtype {rd, imm} => {
                write!(
                    f, "jal     x{}, {}      # Go {} steps: {}",
                    rd, imm << 1, 
                    match imm.is_positive() {
                        true => "forward",
                        false => "backward"
                    },
                    (imm >> 1).abs()
                )
            },
            Instruction::UtypeLUI {rd, imm} => {
               write!(f, "lui     x{}, 0x{:x}", rd, imm >> 12)
            },
            Instruction::UtypeAUIPC {rd, imm} => {
                write!(f, "auipc   x{}, {}", rd, imm)
            },
        }
    }
}