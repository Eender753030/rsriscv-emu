//! The const of magic number for decoding

use crate::exception::Exception;

/// Define const of every instruction corresponding to opcode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    Itype = 0x13,
    ItypeLoad = 0x03,
    ItypeJump = 0x67,
    ItypeFence = 0x0f,
    ItypeSystem = 0x73,
    Rtype = 0x33,
    Stype = 0x23,
    Btype = 0x63,
    Jtype = 0x6f,
    UtypeAuipc = 0x17,
    UtypeLui = 0x37,  
}

impl TryFrom<u8> for OpCode {
    type Error = Exception;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x13 => Ok(OpCode::Itype),
            0x03 => Ok(OpCode::ItypeLoad),
            0x67 => Ok(OpCode::ItypeJump),
            0x0f => Ok(OpCode::ItypeFence),
            0x73 => Ok(OpCode::ItypeSystem),
            0x33 => Ok(OpCode::Rtype),
            0x23 => Ok(OpCode::Stype),
            0x63 => Ok(OpCode::Btype),
            0x6f => Ok(OpCode::Jtype),
            0x17 => Ok(OpCode::UtypeAuipc),
            0x37 => Ok(OpCode::UtypeLui),
            _ => Err(Exception::IllegalInstruction),
        }
    }
}

impl From<OpCode> for u8 {
    fn from(value: OpCode) -> Self {
        value as u8
    }
}

/// Define bits position for decoding
pub trait BitsOp<T> {
    fn get_bits(&self, offset: usize, len: usize) -> Self;

    fn get_bits_signed(&self, offset: usize, len: usize) -> T;
}

impl BitsOp<i32> for u32 {
    fn get_bits(&self, offset: usize, len: usize) -> Self {
        if offset >= 32 || len == 0 {
            return 0;
        }

        let mask = if len >= 32 { !0 } else { !0 >> (32 - len) };

        (*self >> offset) & mask
    }

    fn get_bits_signed(&self, offset: usize, len: usize) -> i32 {
        if offset >= 32 || len == 0 {
            return 0;
        }

        (*self as i32) << (32 - offset - len) >> offset
    }
}

#[cfg(test)]
mod bits_op_test {
    use crate::isa::opcode::BitsOp;

    #[test]
    fn test_get_bits() {
        let raw: u32 = 0x12345678; 
        let ones: u32 = 0xffffffff;
        let a: u32 = 0xaaaaaaaa;
        let beef: u32 = 0xDEADBEEF;

        assert_eq!(raw.get_bits(16, 12), 0x234);
        assert_eq!(ones.get_bits(8, 24), 0xffffff);
        assert_eq!(a.get_bits(1, 31), 0x55555555);
        assert_eq!(beef.get_bits(0, 16), 0xBEEF);
    } 

    #[test]
    fn test_get_bits_signed() {
        let signed: u32 = 0x80000000;
        let unsigned: u32 = 0x7fffffff;
        let f: u32 = 0xffffffff;
        
        assert_eq!(signed.get_bits_signed(1, 31) as u32, 0xc0000000);
        assert_eq!(unsigned.get_bits_signed(1, 31), 0x3fffffff);
        assert_eq!(f.get_bits_signed(31, 1) as u32, 0xffffffff);
    } 
}