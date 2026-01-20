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
