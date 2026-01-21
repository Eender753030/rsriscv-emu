//! Definition of enum corresponding to opcode
mod rv32i;
mod zicsr;
mod zifencei;

pub use rv32i::Rv32iOp;
pub use zicsr::ZicsrOp;
pub use zifencei::ZifenceiOp;

use crate::csr_addr::CsrAddr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstructionData {
    pub rd: u8,
    pub rs1: u8,
    pub rs2: u8,
    pub imm: i32,
}

/// Definition of enum corresponding to opcode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    Base(Rv32iOp, InstructionData),
    Ziscr(ZicsrOp, InstructionData),
    Zifencei(ZifenceiOp, InstructionData),
}

impl Instruction {
    pub fn to_string(&self) -> String {       
        match self {
            Instruction::Base(op, data) => {
                if op.is_itype_ar() {
                    format!("{:<7} x{}, x{}, {}", op, data.rd, data.rs1, data.imm)
                } else if op.is_itype_load() | op.is_itype_jump() {
                    format!("{:<7} x{}, {}(x{})", op, data.rd, data.imm, data.rs1)
                } else if op.is_itype_fence() {
                    match data.imm & 0xf00 {
                        0b0000 => {
                            let succ = check_fence(data.imm);
                            let pred = check_fence(data.imm >> 4);
                            if succ == "iorw" && pred == "iorw" {
                                format!("{:<7}", op)
                            } else {
                                format!("{:<7} {}, {}", op, pred, succ)
                            }
                        },
                        0b1000 => format!("{:<7}", "fence.tso"),
                        _ => format!("{:<7} (Unknown format)", op)
                    }               
                } else if op.is_itype_system() {
                    format!("{:<7}", op)
                } else if op.is_stype() {
                    format!("{:<7} x{}, {}(x{})", op, data.rs2, data.imm, data.rs1)
                } else if op.is_btype() {
                    format!("{:<7} x{}, x{}, {}", op, data.rs1, data.rs2, data.imm)
                } else if op.is_jtype() {
                    format!("{:<7} x{}, {}", op, data.rd, data.imm)
                } else if op.is_utype() {
                    format!("{:<7} x{}, {:#x}", op, data.rd, (data.imm as u32) >> 12)
                } else { // rtype
                    format!("{:<7} x{}, x{}, {}", op, data.rd, data.rs1, data.rs2)
                }
            },
            Instruction::Ziscr(op, data) => {
                if op.is_csr() {
                    let csr_str = CsrAddr::try_from(data.imm)
                        .map(|addr| addr.to_string())
                        .unwrap_or_else(|addr| format!("{:#x}",addr));

                    format!("{:<7} x{}, {}, x{}", op, data.rd, csr_str, data.rs1)
                } else {
                    format!("{:<7}", op)
                }
            },
            Instruction::Zifencei(op, _) => {
                format!("{:<7}", op)
            }
        }
    }
}

fn check_fence(data: i32) -> String {
    let mut output = String::new();
    let mut set = "iorw".chars();

    let mut mask = 0x8;
    
    for _ in 0..4 {
        let mode = set.next().unwrap();
        if (data & mask) == 1 {
            output.push(mode);
        }
        mask >>= 1;
    }

    output
}
