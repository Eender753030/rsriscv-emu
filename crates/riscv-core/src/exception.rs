#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Exception {
    InstructionAddressMisaligned,
    InstructionAccessFault,
    IllegalInstruction(u32),
    Breakpoint,
    LoadAddressMisaligned,
    LoadAccessFault(u32),
    StoreAddressMisaligned,
    EnvironmentCallFromMMode,
}

impl From<Exception> for u32 {
    fn from(value: Exception) -> Self {
        match value {
            Exception::InstructionAddressMisaligned => 0,
            Exception::InstructionAccessFault => 1,
            Exception::IllegalInstruction(_) => 2,
            Exception::Breakpoint => 3,
            Exception::LoadAddressMisaligned => 4,
            Exception::LoadAccessFault(_) => 5,
            Exception::StoreAddressMisaligned => 6,
            Exception::EnvironmentCallFromMMode => 11,
        }
    }
}
