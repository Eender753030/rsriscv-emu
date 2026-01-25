pub struct Alu;

impl Alu {
    pub fn add(data1: u32, data2: u32) -> u32 {
        data1.wrapping_add(data2)
    }

    pub fn add_signed(data1: u32, data2: i32) -> u32 {
        data1.wrapping_add_signed(data2)
    }

    pub fn sub(data1: u32, data2: u32) -> u32 {
        data1.wrapping_sub(data2)
    }

    pub fn xor(data1: u32, data2: u32) -> u32 {
        data1 ^ data2
    }

    pub fn or(data1: u32, data2: u32) -> u32 {
        data1 | data2
    }

    pub fn and(data1: u32, data2: u32) -> u32 {
        data1 & data2
    }

    pub fn shl_logic(data: u32, shift: u32) -> u32 {
        data << (shift % 32)
    }

    pub fn shr_logic(data: u32, shift: u32) -> u32 {
        data >> (shift % 32)
    }

    pub fn shr_ar(data: i32, shift: u32) -> u32 {
        (data >> (shift % 32)) as u32
    }

    pub fn set_less_than(data: i32, cmp_data: i32) -> u32 {
        (data < cmp_data).into()
    }

    pub fn set_less_than_unsigned(data: u32, cmp_data: u32) -> u32 {
        (data < cmp_data).into()
    }

    pub fn mul(data1: u32, data2: u32) -> u32 {
        (data1 as i32 as i64)
            .wrapping_mul(data2 as i32 as i64) as u32
    }

    pub fn mulh(data1: u32, data2: u32) -> u32 {
        ((data1 as i32 as i64)
            .wrapping_mul(data2 as i32 as i64) >> 32) as u32
    }

    pub fn mulh_unsigned(data1: u32, data2: u32) -> u32 {
        ((data1 as u64)
            .wrapping_mul(data2 as u64) >> 32) as u32
    }
    
    pub fn mulh_signed_unsigned(data1: u32, data2: u32) -> u32 {
        ((data1 as i32 as i64)
            .wrapping_mul(data2 as i64) >> 32) as u32
    }
    
    pub fn div(data1: u32, data2: u32) -> u32 {
        if data2 == 0 {
            u32::MAX
        } else {
            (data1 as i32)
                .wrapping_div(data2 as i32) as u32
        }
    }

    pub fn div_unsigned(data1: u32, data2: u32) -> u32 {
        if data2 == 0 {
            u32::MAX
        } else {
            data1.wrapping_div(data2) 
        }
    }

    pub fn rem(data1: u32, data2: u32) -> u32 {
        if data2 == 0 {
            data1
        } else {
            (data1 as i32)
                .wrapping_rem(data2 as i32) as u32
        }
    }

    pub fn rem_unsigned(data1: u32, data2: u32) -> u32 {
        if data2 == 0 {
            data1
        } else {
            data1.wrapping_rem(data2) 
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::Alu;

    #[test]
    fn test_basic_arithmetic() {
        // wrapping add
        assert_eq!(Alu::add(0xFFFFFFFF, 1), 0);
        // wrapping sub
        assert_eq!(Alu::sub(0, 1), 0xFFFFFFFF);
        
        // logic
        assert_eq!(Alu::and(0b1100, 0b1010), 0b1000);
        assert_eq!(Alu::or(0b1100, 0b1010), 0b1110);
        assert_eq!(Alu::xor(0b1100, 0b1010), 0b0110);
    }

    #[test] 
    fn test_shifts() {
        // 1 << 33 should be 1 << 1 = 2
        assert_eq!(Alu::shl_logic(1, 33), 2);

        // Sign extension
        assert_eq!(Alu::shr_ar(-4, 1), -2_i32 as u32);

        // Zero extension
        assert_eq!(Alu::shr_logic(-4_i32 as u32, 1), 0x7FFFFFFE);
    }

    #[test]
    fn test_multiplication() {
        // Simple Mul
        assert_eq!(Alu::mul(10, 20), 200);
    
        // Mulh (Signed * Signed, High bits)
        assert_eq!(Alu::mulh(-1_i32 as u32, -1_i32 as u32), 0);
        
        // Mulhu (Unsigned * Unsigned, High bits)
        assert_eq!(Alu::mulh_unsigned(u32::MAX, 2), 1);

        // Mulhsu (Signed * Unsigned, High bits)
        assert_eq!(Alu::mulh_signed_unsigned(-1_i32 as u32, 2), u32::MAX);
    }

    #[test]
    fn test_division_edge_cases() {
        // Division by Zero
        // RISC-V spec: div by 0 returns -1
        assert_eq!(Alu::div(100, 0), u32::MAX);
        assert_eq!(Alu::div_unsigned(100, 0), u32::MAX);
        
        // RISC-V spec: rem by 0 returns dividend
        assert_eq!(Alu::rem(100, 0), 100);
        assert_eq!(Alu::rem_unsigned(100, 0), 100);

        // Signed Overflow Division
        // RISC-V spec: returns INT_MIN
        let int_min = i32::MIN as u32;
        assert_eq!(Alu::div(int_min, -1_i32 as u32), int_min);
        
        // Remainder should be 0
        assert_eq!(Alu::rem(int_min, -1_i32 as u32), 0);
    }

    #[test]
    fn test_comparisons() {
        // Signed
        assert_eq!(Alu::set_less_than(-1, 1), 1);
        
        // Unsigned
        assert_eq!(Alu::set_less_than_unsigned(-1_i32 as u32, 1), 0);
    }
}