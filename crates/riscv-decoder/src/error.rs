use thiserror::Error;

use crate::opcode::OpCode;

#[derive(Error, Debug, Clone, Copy,PartialEq, Eq)]
pub enum DecodeError {
    #[error("Can not decode opcode: {0:#04x}")]
    UnknownOpcode(u8),

    #[error("Unknown Instruction from opcode: {0} Raw data: {1:#010x}")]
    UnknownInstruction(OpCode, u32),

    #[cfg(feature = "c")]
    #[error("Not a Compress Instruction from opcode: 0b11")]
    NotCompressInstruction,

    #[cfg(feature = "c")]
    #[error("Unknown Compress Instruction from opcode: {0:#04b} Raw data: {1:#06x}")]
    UnknownCompressInstruction(u8, u16)
}