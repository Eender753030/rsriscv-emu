pub struct Branch;

impl Branch {
    pub fn equal(data1: u32, data2: u32) -> bool {
        data1 == data2
    }

    pub fn not_equal(data1: u32, data2: u32) -> bool {
        data1 != data2
    }

    pub fn less(data1: i32, data2: i32) -> bool {
        data1 < data2
    }

    pub fn less_unsigned(data1: u32, data2: u32) -> bool {
        data1 < data2
    }

    pub fn greater_eqaul(data1: i32, data2: i32) -> bool {
        data1 >= data2
    }

    pub fn greater_eqaul_unsigned(data1: u32, data2: u32) -> bool {
        data1 >= data2
    }
}

#[cfg(test)]
mod tests {
    use super::Branch;

    #[test]
    fn test_equality() {
        assert!(Branch::equal(0xDEADBEEF, 0xDEADBEEF));
        assert!(Branch::not_equal(0xDEADBEEF, 0xCAFEBABE));
    }

    #[test]
    fn test_signed_comparison() {
        // 10 < 20
        assert!(Branch::less(10, 20));
        // -20 < 10
        assert!(Branch::less(-20, 10));
        // -20 < -10
        assert!(Branch::less(-20, -10));
        
        // 10 > -100
        assert!(Branch::greater_eqaul(10, -100));
        assert!(!Branch::less(10, -100));
    }

    #[test]
    fn test_unsigned_comparison() {
        let small = 10u32;
        let big = 20u32;
        let huge = u32::MAX;

        assert!(Branch::less_unsigned(small, big));
        assert!(Branch::less_unsigned(big, huge));
        
        assert!(Branch::less_unsigned(10, 0xFFFFFFFF));
        assert!(!Branch::less(10, -1_i32)); 
    }
}