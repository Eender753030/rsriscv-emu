use Exception::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exception {
    InstructionAddressMisaligned,
    InstructionAccessFault(u32),
    IllegalInstruction(u32),
    Breakpoint,
    LoadAddressMisaligned,
    LoadAccessFault(u32),
    StoreOrAmoAddressMisaligned,
    StoreOrAmoAccessFault(u32),
    EnvironmentCallFromUMode,
    EnvironmentCallFromSMode,
    EnvironmentCallFromMMode,
    InstructionPageFault(u32),
    LoadPageFault(u32),
    StoreOrAmoPageFault(u32),
    
    #[cfg(not(feature = "zicsr"))] Ecall, // Custom
    #[cfg(not(feature = "zicsr"))] Ebreak, // Custom
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
            StoreOrAmoAddressMisaligned  => 6,
            StoreOrAmoAccessFault(_)     => 7,
            EnvironmentCallFromUMode     => 8,
            EnvironmentCallFromSMode     => 9,
            EnvironmentCallFromMMode     => 11,
            InstructionPageFault(_)      => 12,
            LoadPageFault(_)             => 13,
            StoreOrAmoPageFault(_)       => 15,
        
            #[cfg(not(feature = "zicsr"))] Ecall  => 100,
            #[cfg(not(feature = "zicsr"))] Ebreak => 101,
        }
    }
}

impl std::fmt::Display for Exception {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstructionAddressMisaligned => f.write_str(" 0: Instruction Address Misaligned"),
            Breakpoint                   => f.write_str(" 3: Breakpoint"),
            LoadAddressMisaligned        => f.write_str(" 4: Load Address Misaligned"),
            StoreOrAmoAddressMisaligned       => f.write_str(" 6: Store Address Misaligned"),  
            EnvironmentCallFromUMode     => f.write_str(" 8: Environment Call From U-Mode"),
            EnvironmentCallFromSMode     => f.write_str(" 9: Environment Call From S-Mode"),
            EnvironmentCallFromMMode     => f.write_str("11: Environment Call From M-Mode"),

            InstructionAccessFault(addr) => write!(f, " 1: Instruction Access Fault (From: {:#010x})", addr),
            IllegalInstruction(raw)      => write!(f, " 2: Illegal Instruction (Raw: {:#010x})", raw),
            LoadAccessFault(addr)        => write!(f, " 5: Load Access Fault (From: {:#010x})", addr),
            StoreOrAmoAccessFault(addr)       => write!(f, " 7: Store Access Fault (From: {:#010x})", addr),
            InstructionPageFault(addr)   => write!(f, "12: Instruction Page Fault (From: {:#010x})", addr),
            LoadPageFault(addr)          => write!(f, "13: Load Page Fault (From: {:#010x})", addr),
            StoreOrAmoPageFault(addr)    => write!(f, "15: Store/AMO Page Fault (From: {:#010x})", addr),

            #[cfg(not(feature = "zicsr"))] Ecall  => f.write_str("100(Custom): Ecall"),
            #[cfg(not(feature = "zicsr"))] Ebreak => f.write_str("101(Custom): Ebreak"),
        }
    }
}
