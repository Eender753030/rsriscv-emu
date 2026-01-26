//! The const of magic number for decoding

use crate::error::DecodeError;

use OpCode::*;

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
        Ok(match value {
            0x13 => ItypeAr,
            0x03 => ItypeLoad,
            0x67 => ItypeJump,
            0x0f => ItypeFence,
            0x73 => ItypeSystem,
            0x33 => Rtype,
            0x23 => Stype,
            0x63 => Btype,
            0x6f => Jtype,
            0x37 => UtypeLui,
            0x17 => UtypeAuipc,       
            _    => return Err(DecodeError::UnknownOpcode(value)),
        })
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
            ItypeAr     => "I-type",
            ItypeLoad   => "I-type: load",
            ItypeJump   => "I-type: jump",
            ItypeFence  => "I-type: fence",
            ItypeSystem => "I-type: system",
            Rtype       => "R-type",
            Stype       => "S-type",
            Btype       => "B-type",
            Jtype       => "J-type",
            UtypeLui    => "U-type: lui",
            UtypeAuipc  => "U-type: auipc",        
        };
        
        f.pad(&format!("{:#02x}({})", opcode, op_str))
    }
}