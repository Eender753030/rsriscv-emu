use thiserror::Error;

use crate::opcode::OpCode;

#[derive(Error, Debug, Clone, Copy,PartialEq, Eq)]
pub enum DecodeError {
    #[error("Can not decode opcode: {0:#02x}")]
    UnknownOpcode(u8),

    #[error("Unknown Instruction from opcode: {0} Raw data: {1:#08x}")]
    UnknownInstruction(OpCode, u32)
}