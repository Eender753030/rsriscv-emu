pub trait DebugInterface {
    fn inspect_regs(&self) -> [u32; 32];

    fn inspect_pc(&self) -> u32;

    fn inspect_csrs(&self) -> Vec<(String, u32)>;

    fn inspect_ins(&self, start: u32, count: usize) -> Vec<(u32, String)>;

    fn inspect_mem(&self, start: u32, len: usize) -> Vec<u8>;
}