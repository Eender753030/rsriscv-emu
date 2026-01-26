use Exception::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exception {
    InstructionAddressMisaligned,
    InstructionAccessFault(u32),
    IllegalInstruction(u32),
    Breakpoint,
    LoadAddressMisaligned,
    LoadAccessFault(u32),
    StoreAddressMisaligned,
    StoreAccessFault(u32),
    EnvironmentCallFromUMode,
    EnvironmentCallFromSMode,
    EnvironmentCallFromMMode,
    InstructionPageFault(u32),
    LoadPageFault(u32),
    StoreOrAmoPageFault(u32),
}

impl From<Exception> for u32 {
    fn from(value: Exception) -> Self {
        match value {
            InstructionAddressMisaligned => 0,
            InstructionAccessFault(_)    => 1,
            IllegalInstruction(_)        => 2,
            Breakpoint                   => 3,
            LoadAddressMisaligned        => 4,
            LoadAccessFault(_)           => 5,
            StoreAddressMisaligned       => 6,
            StoreAccessFault(_)          => 7,
            EnvironmentCallFromUMode     => 8,
            EnvironmentCallFromSMode     => 9,
            EnvironmentCallFromMMode     => 11,
            InstructionPageFault(_)      => 12,
            LoadPageFault(_)             => 13,
            StoreOrAmoPageFault(_)       => 15,
        }
    }
}
