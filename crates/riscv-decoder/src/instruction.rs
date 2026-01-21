//! Definition of enum corresponding to opcode
mod rv32i;
mod zicsr;
mod zifencei;

pub use rv32i::Rv32iOp;
pub use zicsr::ZicsrOp;
pub use zifencei::ZifenceiOp;

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
