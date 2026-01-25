//! Define bits position operation for decoding

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
mod tests {
    use crate::bits_op::BitsOp;

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