use modular_bitfield::prelude::*;
use crate::exception::Exception;

#[bitfield]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Mstate {
    #[skip] __: B2,
    mie: B1,
    #[skip] __: B3,
    mpie: B1,
    #[skip] __: B25,
}

impl Mstate {
    fn reset(&mut self) {
        self.set_mie(0);
        self.set_mpie(0);
    }
}

impl From<Mstate> for u32 {
    fn from(value: Mstate) -> Self {
        u32::from_le_bytes(value.into_bytes())
    }
}

impl Default for Mstate {
    fn default() -> Self {
        Mstate::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CsrFile {
    mstate: Mstate,
    mie: u32,
    mtvec: u32,
    mepc: u32,
    mcause: u32,
    mnscratch: u32,
}

const MIE_MASK: u32 = 1 << 3;
const MPIE_MASK: u32 = 1 << 7;

impl CsrFile {
    pub fn read(&mut self, addr: u16) -> Result<u32, Exception> {
        Ok(match CsrAddr::try_from(addr)? {
            CsrAddr::Ustatus => 0,
            CsrAddr::Satp => 0,
            CsrAddr::Mstatus => self.mstate.into(),
            CsrAddr::Medeleg => 0,
            CsrAddr::Mideleg => 0,
            CsrAddr::Mie => self.mie,
            CsrAddr::Mtvec => self.mtvec,
            CsrAddr::Mepc => self.mepc,
            CsrAddr::Mcause => self.mcause,
            CsrAddr::Pmpcfg0 => 0,
            CsrAddr::Pmpaddr0 => 0,
            CsrAddr::Mnscratch => self.mnscratch,
            CsrAddr::Mhartid => 0,
        })
    }

    pub fn write(&mut self, addr: u16, data: u32) -> Result<(), Exception> {
        match CsrAddr::try_from(addr)? {
            CsrAddr::Ustatus => {},
            CsrAddr::Satp => {},
            CsrAddr::Mstatus => {
                let mie = ((data & MIE_MASK) >> 3) as u8;
                let mpie = ((data & MPIE_MASK) >> 7) as u8;
                self.mstate.set_mie(mie);
                self.mstate.set_mpie(mpie);
            },
            CsrAddr::Medeleg => {},
            CsrAddr::Mideleg => {},
            CsrAddr::Mie => self.mie = data,
            CsrAddr::Mtvec => self.mtvec = data,
            CsrAddr::Mepc => self.mepc = data,
            CsrAddr::Mcause => self.mcause = data,
            CsrAddr::Pmpcfg0 => {},
            CsrAddr::Pmpaddr0 => {},
            CsrAddr::Mnscratch => self.mnscratch = data, 
            CsrAddr::Mhartid => return Err(Exception::IllegalInstruction),
        };

        Ok(())
    }

    pub fn trap_entry(&mut self, curr_pc: u32, except_code: Exception) -> u32 {
        self.mepc = curr_pc;
        self.mcause = except_code.into();
        self.mstate.set_mpie(self.mstate.mie());
        self.mstate.set_mie(0);
        self.mtvec
    } 

    pub fn trap_ret(&mut self) -> u32 {
        self.mstate.set_mie(self.mstate.mpie());
        self.mstate.set_mpie(1);
        self.mepc
    } 

    pub fn reset(&mut self) {
        self.mstate.reset();
        self.mtvec = 0;
        self.mepc = 0;
        self.mcause = 0;
    }

    pub fn inspect(&self) -> Vec<(String, u32)> {
        vec![
            ("mstatus".to_string(), self.mstate.into()),
            ("mtvec".to_string(), self.mtvec),
            ("mepc".to_string(), self.mepc),
            ("mcause".to_string(), self.mcause),
            ("mhartid".to_string(), 0),
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CsrAddr {
    Ustatus = 0x000,
    Satp = 0x180,
    Mstatus = 0x300,
    Medeleg = 0x302,
    Mideleg = 0x303,
    Mie = 0x304,
    Mtvec = 0x305,
    Mepc = 0x341,
    Mcause = 0x342,
    Pmpcfg0 = 0x3a0,
    Pmpaddr0 = 0x3b0,
    Mnscratch = 0x744,
    Mhartid = 0xf14,
}

impl TryFrom<u16> for CsrAddr {
    type Error = Exception;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x000 => Ok(CsrAddr::Ustatus),
            0x180 => Ok(CsrAddr::Satp),
            0x300 => Ok(CsrAddr::Mstatus),
            0x302 => Ok(CsrAddr::Medeleg),
            0x303 => Ok(CsrAddr::Mideleg),
            0x304 => Ok(CsrAddr::Mie),
            0x305 => Ok(CsrAddr::Mtvec),
            0x341 => Ok(CsrAddr::Mepc),
            0x342 => Ok(CsrAddr::Mcause),
            0x3a0 => Ok(CsrAddr::Pmpcfg0),
            0x3b0 => Ok(CsrAddr::Pmpaddr0),
            0x744 => Ok(CsrAddr::Mnscratch),
            0xf14 => Ok(CsrAddr::Mhartid),   
            _ => Err(Exception::IllegalInstruction),
        }
    }
}
