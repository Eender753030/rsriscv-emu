use crate::Exception;

use CsrAddr::*;

use super::PMPCFG_NUM;

const PMPCFG_END: u16 = 0x3a0 + PMPCFG_NUM as u16;
const PMPADDR_END: u16 = 0x3b0 + (PMPCFG_NUM * 4) as u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CsrAddr {
    Ustatus,

    Sstatus,
    Sie,
    Stvec,
    Sscratch,
    Sepc,
    Scause,
    Stval,
    Sip,
    Satp,

    Mstatus,
    Medeleg,
    Mideleg,
    Mie,
    Mtvec,
    Mscratch,
    Mepc,
    Mcause,
    Mtval,
    Mip,
    Pmpcfg(usize),
    Pmpaddr(usize),
    Mhartid,
}

impl TryFrom<u16> for CsrAddr {
    type Error = Exception;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(match value {
            0x000 => Ustatus,

            0x100 => Sstatus,
            0x104 => Sie,
            0x105 => Stvec,
            0x140 => Sscratch,
            0x141 => Sepc,
            0x142 => Scause,
            0x143 => Stval,
            0x144 => Sip,
            0x180 => Satp,
            
            0x300 => Mstatus,
            0x302 => Medeleg,
            0x303 => Mideleg,
            0x304 => Mie,
            0x305 => Mtvec,
            0x340 => Mscratch,
            0x341 => Mepc,
            0x342 => Mcause,
            0x343 => Mtval,
            0x344 => Mip,
            num @ 0x3a0..=PMPCFG_END => Pmpcfg((num - 0x3a0) as usize),
            num @ 0x3b0..=PMPADDR_END => Pmpaddr((num - 0x3b0) as usize),
            0xf14 => Mhartid, 

            _     => return Err(Exception::IllegalInstruction(value as u32)),
        })
    }
}
