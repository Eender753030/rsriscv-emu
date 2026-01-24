use crate::exception::Exception;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CsrAddr {
    Ustatus = 0x000,

    Sstatus = 0x100,
    Sie = 0x104,
    Stvec = 0x105,
    Sscratch = 0x140,
    Sepc = 0x141,
    Scause = 0x142,
    Stval = 0x143,
    Sip = 0x144,
    Satp = 0x180,

    Mstatus = 0x300,
    Medeleg = 0x302,
    Mideleg = 0x303,
    Mie = 0x304,
    Mtvec = 0x305,
    Mscratch = 0x340,
    Mepc = 0x341,
    Mcause = 0x342,
    Mtval = 0x343,
    Mip = 0x344,
    Pmpcfg0 = 0x3a0,
    Pmpaddr0 = 0x3b0,
    Mhartid = 0xf14,
}

impl TryFrom<u16> for CsrAddr {
    type Error = Exception;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x000 => Ok(CsrAddr::Ustatus),

            0x100 => Ok(CsrAddr::Sstatus),
            0x104 => Ok(CsrAddr::Sie),
            0x105 => Ok(CsrAddr::Stvec),
            0x140 => Ok(CsrAddr::Sscratch),
            0x141 => Ok(CsrAddr::Sepc),
            0x142 => Ok(CsrAddr::Scause),
            0x143 => Ok(CsrAddr::Stval),
            0x144 => Ok(CsrAddr::Sip),
            0x180 => Ok(CsrAddr::Satp),
            
            0x300 => Ok(CsrAddr::Mstatus),
            0x302 => Ok(CsrAddr::Medeleg),
            0x303 => Ok(CsrAddr::Mideleg),
            0x304 => Ok(CsrAddr::Mie),
            0x305 => Ok(CsrAddr::Mtvec),
            0x340 => Ok(CsrAddr::Mscratch),
            0x341 => Ok(CsrAddr::Mepc),
            0x342 => Ok(CsrAddr::Mcause),
            0x343 => Ok(CsrAddr::Mtval),
            0x344 => Ok(CsrAddr::Mip),
            0x3a0 => Ok(CsrAddr::Pmpcfg0),
            0x3b0 => Ok(CsrAddr::Pmpaddr0),
            0xf14 => Ok(CsrAddr::Mhartid), 

            _ => Err(Exception::IllegalInstruction(value as u32)),
        }
    }
}