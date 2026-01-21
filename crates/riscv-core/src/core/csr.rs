use crate::exception::Exception;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CsrFile {
    mstate: u32,
    mtvec: u32,
    mepc: u32,
    mcause: u32,
}

const MIE_MASK: u32 = 1 << 3;
const MPIE_MASK: u32 = 1 << 7;

impl CsrFile {
    pub fn read(&mut self, addr: u16) -> Result<u32, Exception> {
        Ok(match CsrAddr::try_from(addr)? {
            CsrAddr::Mstatus => {
                let mask = MIE_MASK | MPIE_MASK;
                self.mstate & mask
            }
            CsrAddr::Mtvec => self.mtvec,
            CsrAddr::Mepc => self.mepc,
            CsrAddr::Mcause => self.mcause,
        })
    }

    pub fn write(&mut self, addr: u16, data: u32) -> Result<(), Exception> {
        match CsrAddr::try_from(addr)? {
            CsrAddr::Mstatus => {
                let mask = MIE_MASK | MPIE_MASK;
                self.mstate = (self.mstate & !mask) | (data & mask);
            },
            CsrAddr::Mtvec => self.mtvec = data,
            CsrAddr::Mepc => self.mepc = data,
            CsrAddr::Mcause => self.mcause = data,
        };

        Ok(())
    }

    pub fn trap_entry(&mut self, curr_pc: u32, except_code: Exception) -> u32 {
        self.mepc = curr_pc;
        self.mcause = except_code.into();
        let mask = MIE_MASK | MPIE_MASK;
        self.mstate = (self.mstate & !mask) | ((self.mstate & MIE_MASK) << 4) & (!MIE_MASK);
        self.mtvec
    } 

    pub fn trap_ret(&mut self) -> u32 {
        let mask = MIE_MASK | MPIE_MASK;
        self.mstate = (self.mstate & !mask) | ((self.mstate & MPIE_MASK) >> 4) | MPIE_MASK;
        self.mepc
    } 

}

pub enum CsrAddr {
    Mstatus = 0x300,
    Mtvec = 0x305,
    Mepc = 0x341,
    Mcause = 0x342,
}

impl TryFrom<u16> for CsrAddr {
    type Error = Exception;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x300 => Ok(CsrAddr::Mstatus),
            0x305 => Ok(CsrAddr::Mtvec),
            0x341 => Ok(CsrAddr::Mepc),
            0x342 => Ok(CsrAddr::Mcause),
            _ => Err(Exception::IllegalInstruction),
        }
    }
}
