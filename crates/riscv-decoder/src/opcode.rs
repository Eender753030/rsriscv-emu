//! The const of magic number for decoding

use crate::error::DecodeError;

/// Define const of every instruction corresponding to opcode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    ItypeAr = 0x13,
    ItypeLoad = 0x03,
    ItypeJump = 0x67,
    ItypeFence = 0x0f,
    ItypeSystem = 0x73,
    Rtype = 0x33,
    Stype = 0x23,
    Btype = 0x63,
    Jtype = 0x6f,
    UtypeLui = 0x37,  
    UtypeAuipc = 0x17,
}

impl TryFrom<u8> for OpCode {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x13 => Ok(OpCode::ItypeAr),
            0x03 => Ok(OpCode::ItypeLoad),
            0x67 => Ok(OpCode::ItypeJump),
            0x0f => Ok(OpCode::ItypeFence),
            0x73 => Ok(OpCode::ItypeSystem),
            0x33 => Ok(OpCode::Rtype),
            0x23 => Ok(OpCode::Stype),
            0x63 => Ok(OpCode::Btype),
            0x6f => Ok(OpCode::Jtype),
            0x37 => Ok(OpCode::UtypeLui),
            0x17 => Ok(OpCode::UtypeAuipc),       
            _ => Err(DecodeError::UnknownOpcode(value)),
        }
    }
}

impl From<OpCode> for u8 {
    fn from(value: OpCode) -> Self {
        value as u8
    }
}

impl std::fmt::Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let opcode: u8 = (*self).into();
        let op_str = match self {
            OpCode::ItypeAr => "I-type",
            OpCode::ItypeLoad => "I-type: load",
            OpCode::ItypeJump => "I-type: jump",
            OpCode::ItypeFence => "I-type: fence",
            OpCode::ItypeSystem => "I-type: system",
            OpCode::Rtype => "R-type",
            OpCode::Stype => "S-type",
            OpCode::Btype => "B-type",
            OpCode::Jtype => "J-type",
            OpCode::UtypeLui => "U-type: lui",
            OpCode::UtypeAuipc => "U-type: auipc",        
        };
        
        f.pad(&format!("{:#02x}({})", opcode, op_str))
    }
}