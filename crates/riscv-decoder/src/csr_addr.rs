pub enum CsrAddr {
    Mstatus = 0x300,
    Mtvec = 0x305,
    Mepc = 0x341,
    Mcause = 0x342,
}

impl std::fmt::Display for CsrAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad( 
            match self {
                CsrAddr::Mstatus => "mstatus",
                CsrAddr::Mtvec => "mtvec",
                CsrAddr::Mepc => "mepc",
                CsrAddr::Mcause => "mcause",
            }
        )
    }
}

impl TryFrom<i32> for CsrAddr {
    type Error = i32;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0x300 => Ok(CsrAddr::Mstatus),
            0x305 => Ok(CsrAddr::Mtvec),
            0x341 => Ok(CsrAddr::Mepc),
            0x342 => Ok(CsrAddr::Mcause),
            _ => Err(value),
        }
    }
}