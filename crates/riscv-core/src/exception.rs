#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Exception {
    InstructionAddressMisaligned = 0,
    InstructionAccessFault = 1,
    IllegalInstruction = 2,
    Breakpoint = 3,
    LoadAddressMisaligned = 4,
    LoadAccessFault = 5,
    StoreAddressMisaligned = 6,
    EnvironmentCallFromMMode = 11,
}

impl From<Exception> for u32 {
    fn from(value: Exception) -> Self {
        value as u32
    }
}
