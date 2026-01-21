use thiserror::Error;

use crate::opcode::OpCode;

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("Can not decode opcode: {0:#02X}")]
    UnknownOpcode(u8),

    #[error("Unknown Instruction from opcode: {0} Raw data: {1:#08X}")]
    UnknownInstruction(OpCode, u32)
}