use modular_bitfield::prelude::*;
use crate::{core::privilege::PrivilegeMode, exception::Exception};

#[bitfield]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Mstatus {
    #[skip] __: B1, 
    sie: B1, // bit 1
    #[skip] __: B1,
    mie: B1, // bit 3
    #[skip] __: B1,
    spie: B1, // bit 5
    #[skip] __: B1,
    mpie: B1, // bit 7
    spp: B1, // bit 8
    #[skip] __: B2,
    mpp: B2, // bit 12:11
    #[skip] __: B19,
}

const M_MODE_MASK: u32 = (1 << 1) | (1 << 3) | (1 << 5) | (3 << 7) | (3 << 11);
const S_MODE_MASK: u32 = (1 << 1) | (1 << 5) | (1 << 8);

impl Mstatus {
    fn read_m(&self) -> u32{
        u32::from(*self)
    }

    fn read_s(&self) -> u32{
        u32::from(*self) & S_MODE_MASK
    }

    fn write_m(&mut self, data: u32) {
        *self = ((u32::from(*self) & !M_MODE_MASK) | (data & M_MODE_MASK)).into();
    }

    fn write_s(&mut self, data: u32) {
        *self = ((u32::from(*self) & !S_MODE_MASK) | (data & S_MODE_MASK)).into();
    }


    fn reset(&mut self) {
        *self = Self::default()
    }
}

impl From<Mstatus> for u32 {
    fn from(value: Mstatus) -> Self {
        u32::from_le_bytes(value.into_bytes())
    }
}

impl From<u32> for Mstatus {
    fn from(value: u32) -> Self {
        Mstatus::from_bytes(value.to_le_bytes())
    }
}

impl Default for Mstatus {
    fn default() -> Self {
        Mstatus::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CsrFile {
    stvec: u32,
    sepc: u32,
    scause: u32,
    sscratch: u32,
    stval: u32,
    satp: u32,

    mstatus: Mstatus,
    medeleg: u32,
    mideleg: u32,
    mie: u32,
    mtvec: u32,
    mscratch: u32,
    mepc: u32,
    mcause: u32,
    mtval: u32,
    mip: u32,
}

const MODE_MASK: u16 = 3 << 8;
// const INTERRUPT_MASK: u32 = 1 << 31;

impl CsrFile {
    pub fn read(&mut self, addr: u16, mode: PrivilegeMode) -> Result<u32, Exception> {    
        if (mode as u16) < ((addr & MODE_MASK) >> 8) {
            Err(Exception::IllegalInstruction(addr as u32))
        } else {
            Ok(match CsrAddr::try_from(addr)? {
                CsrAddr::Ustatus => 0,

                CsrAddr::Sstatus => self.mstatus.read_s(),
                CsrAddr::Sie => self.mie & self.mideleg,
                CsrAddr::Stvec => self.stvec,
                CsrAddr::Sscratch => self.sscratch,
                CsrAddr::Sepc => self.sepc,
                CsrAddr::Scause => self.scause,
                CsrAddr::Stval => self.stval,
                CsrAddr::Sip => self.mip & self.mideleg,
                CsrAddr::Satp => self.satp,

                CsrAddr::Mstatus => self.mstatus.read_m(),
                CsrAddr::Medeleg => self.medeleg,
                CsrAddr::Mideleg => self.mideleg,
                CsrAddr::Mie => self.mie,
                CsrAddr::Mtvec => self.mtvec,
                CsrAddr::Mscratch => self.mscratch,
                CsrAddr::Mepc => self.mepc,
                CsrAddr::Mcause => self.mcause,
                CsrAddr::Mtval => self.mtval,
                CsrAddr::Mip => self.mip,
                CsrAddr::Pmpcfg0 => 0,
                CsrAddr::Pmpaddr0 => 0,
                CsrAddr::Mhartid => 0,
            })
        }
    }

    pub fn write(&mut self, addr: u16, data: u32, mode: PrivilegeMode) -> Result<(), Exception> {
        if (mode as u16) < ((addr & MODE_MASK) >> 8) {
            Err(Exception::IllegalInstruction(addr as u32))
        } else {
            match CsrAddr::try_from(addr)? {
                CsrAddr::Ustatus => {},

                CsrAddr::Sstatus => self.mstatus.write_s(data),
                CsrAddr::Sie => self.mie = (self.mie & !self.mideleg) | (data & self.mideleg),
                CsrAddr::Stvec => self.stvec = data,
                CsrAddr::Sscratch => self.sscratch = data,
                CsrAddr::Sepc => self.sepc = data,
                CsrAddr::Scause => self.scause = data,
                CsrAddr::Stval => self.stval = data,
                CsrAddr::Sip => self.mip = (self.mip & !self.mideleg) | (data & self.mideleg),
                CsrAddr::Satp => self.satp = data,

                CsrAddr::Mstatus => self.mstatus.write_m(data),
                CsrAddr::Medeleg => self.medeleg = data,
                CsrAddr::Mideleg => self.mideleg = data,
                CsrAddr::Mie => self.mie = data,
                CsrAddr::Mtvec => self.mtvec = data,
                CsrAddr::Mscratch => self.mscratch = data,
                CsrAddr::Mepc => self.mepc = data,
                CsrAddr::Mcause => self.mcause = data,
                CsrAddr::Mtval => self.mtval = data,
                CsrAddr::Mip => self.mip = data,
                CsrAddr::Pmpcfg0 => {},
                CsrAddr::Pmpaddr0 => {}, 
                CsrAddr::Mhartid => return Err(Exception::IllegalInstruction(addr as u32)),
            };
            Ok(())
        }
    }

    pub fn trap_entry(&mut self, curr_pc: u32, except_code: Exception, mode: PrivilegeMode) -> (PrivilegeMode, u32) {
        let target_mode = match mode {
            PrivilegeMode::Machine => PrivilegeMode::Machine,
            PrivilegeMode::Supervisor | PrivilegeMode::User => {
                if self.medeleg & (1 << (u32::from(except_code))) > 0 {
                    PrivilegeMode::Supervisor
                } else {
                    PrivilegeMode::Machine
                }
            }
        };

        let tval = match except_code {
            Exception::IllegalInstruction(raw) => raw,
            Exception::LoadAccessFault(addr) => addr,
            _ => 0
        };
   
        match target_mode {
            PrivilegeMode::Machine => {
                self.mepc = curr_pc;
                self.mcause = except_code.into();
                self.mtval = tval;
                self.mstatus.set_mpie(self.mstatus.mie());
                self.mstatus.set_mie(0);
                self.mstatus.set_mpp(mode as u8);
                let base_addr = self.mtvec & !0b11;
                (target_mode,
                if self.mtvec & 0b11 == 0b01 {
                    base_addr + 4 * u32::from(except_code)
                } else {
                    base_addr
                })
            },
            PrivilegeMode::Supervisor => {
                self.sepc = curr_pc;
                self.scause = except_code.into();
                self.stval = tval;
                self.mstatus.set_spie(self.mstatus.sie());
                self.mstatus.set_sie(0);
                self.mstatus.set_spp(mode as u8);
                let base_addr = self.stvec & !0b11;
                (target_mode,
                if self.stvec & 0b11 == 0b01 {
                    base_addr + 4 * u32::from(except_code)
                } else {
                    base_addr
                })
            },
            PrivilegeMode::User => {(PrivilegeMode::User, 0)},
        }
    } 

    pub fn trap_mret(&mut self) -> (PrivilegeMode, u32) {
        let mode = self.mstatus.mpp().into();
        
        self.mstatus.set_mie(self.mstatus.mpie());
        self.mstatus.set_mpie(1);
        self.mstatus.set_mpp(0);

        (mode, self.mepc)
    } 

    pub fn trap_sret(&mut self) -> (PrivilegeMode, u32) {
        let mode = match self.mstatus.spp() {
            0b0 => PrivilegeMode::User,
            _ => PrivilegeMode::Supervisor,
        };
        
        self.mstatus.set_sie(self.mstatus.spie());
        self.mstatus.set_spie(1);
        self.mstatus.set_spp(0);

        (mode, self.sepc)
    } 

    pub fn reset(&mut self) {
        self.mstatus.reset();
        self.mtvec = 0;
        self.mepc = 0;
        self.mcause = 0;
    }

    pub fn inspect(&self) -> Vec<(String, u32)> {
        vec![
            ("ustatus".to_string(), 0),
            ("sstatus".to_string(), self.mstatus.read_s()),
            ("sie".to_string(), self.mie & self.mideleg),
            ("stvec".to_string(), self.stvec),
            ("sscratch".to_string(), self.sscratch),
            ("sepc".to_string(), self.sepc),
            ("scause".to_string(), self.scause),
            ("stval".to_string(), self.stval),
            ("sip".to_string(), self.mip & self.mideleg),
            ("stap".to_string(), self.satp),
            ("mstatus".to_string(), self.mstatus.read_m()),
            ("medeleg".to_string(), self.medeleg),
            ("mideleg".to_string(), self.mideleg),
            ("mie".to_string(), self.mie),
            ("mtvec".to_string(), self.mtvec),
            ("mscratch".to_string(), self.mscratch),
            ("mepc".to_string(), self.mepc),
            ("mcause".to_string(), self.mcause),
            ("mip".to_string(), self.mip),
            ("pmpcfg0".to_string(), 0),
            ("pmpaddr0".to_string(), 0),
            ("mhartid".to_string(), 0),
        ]
    }
}

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
