mod mstatus;
mod satp;
mod addr;

use crate::{core::privilege::PrivilegeMode, exception::Exception};
use mstatus::Mstatus;
use satp::Satp;
use addr::CsrAddr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CsrFile {
    stvec: u32,
    sepc: u32,
    scause: u32,
    sscratch: u32,
    stval: u32,
    satp: Satp,

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
                CsrAddr::Satp => self.satp.into(),

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
                CsrAddr::Satp => self.satp = data.into(),

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
            Exception::LoadAccessFault(addr) |
            Exception::StoreAccessFault(addr) |
            Exception::InstructionAccessFault(addr) | 
            Exception::LoadPageFault(addr) |
            Exception::StoreOrAmoPageFault(addr) |
            Exception::InstructionPageFault(addr)
            => addr,
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
        *self = Self::default()
    }

    pub fn check_satp(&self) -> Option<u32> {
        if self.satp.mode() > 0  {
            Some(self.satp.ppn())
        } else {
            None
        }
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
            ("stap".to_string(), self.satp.into()),
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

#[cfg(test)]
mod tests {
    use crate::core::CsrFile;
    use crate::core::privilege::PrivilegeMode;
    use crate::exception::Exception;

    #[test]
    fn test_csr_rw_permission() {
        let mut csr = CsrFile::default();
        let val = 0xDEAD_BEEF;

        assert!(csr.write(0x340, val, PrivilegeMode::Machine).is_ok());
        assert_eq!(csr.read(0x340, PrivilegeMode::Machine), Ok(val));
 
        assert_eq!(
            csr.read(0x340, PrivilegeMode::Supervisor),
            Err(Exception::IllegalInstruction(0x340))
        );

        assert_eq!(
            csr.write(0x340, 0x1234, PrivilegeMode::User),
            Err(Exception::IllegalInstruction(0x340))
        );
    }

    #[test]
    fn test_mstatus_behavior() {
        let mut csr = CsrFile::default();
        
        let pattern = (1 << 3) | (1 << 7); 
        csr.write(0x300, pattern, PrivilegeMode::Machine).unwrap();
        
        let read_back = csr.read(0x300, PrivilegeMode::Machine).unwrap();
        assert_eq!(read_back & pattern, pattern);
        let sstatus = csr.read(0x100, PrivilegeMode::Supervisor).unwrap();
        assert_eq!(sstatus, 0);
        
    }

    #[test]
    fn test_trap_entry() {
        let mut csr = CsrFile::default();
        let fault_pc = 0x8000_1000;
        let cause = Exception::IllegalInstruction(0);
        
        let mstatus_init = 1 << 3;
        csr.write(0x300, mstatus_init, PrivilegeMode::Machine).unwrap();
        
        let handler_base = 0x8000_0004;
        csr.write(0x305, handler_base, PrivilegeMode::Machine).unwrap();

        let (next_mode, next_pc) = csr.trap_entry(fault_pc, cause, PrivilegeMode::Machine);

        assert_eq!(next_mode, PrivilegeMode::Machine);
        
        assert_eq!(next_pc, handler_base);

        assert_eq!(csr.mepc, fault_pc);
        assert_eq!(csr.mcause, u32::from(cause));

        let mstatus_new = csr.read(0x300, PrivilegeMode::Machine).unwrap();
        assert_eq!(mstatus_new & (1 << 3), 0);
        assert_eq!(mstatus_new & (1 << 7), (1 << 7));
    }

    #[test]
    fn test_trap_return_mret() {
        let mut csr = CsrFile::default();
        let ret_pc = 0x8000_2000;

        csr.mepc = ret_pc;
        let mstatus_trap_state = (1 << 7) | (3 << 11); 
        csr.write(0x300, mstatus_trap_state, PrivilegeMode::Machine).unwrap();

        let (ret_mode, target_pc) = csr.trap_mret();

        assert_eq!(target_pc, ret_pc);
        assert_eq!(ret_mode, PrivilegeMode::Machine);

        let mstatus_after = csr.read(0x300, PrivilegeMode::Machine).unwrap();
        assert_eq!(mstatus_after & (1 << 3), (1 << 3));
        assert_eq!(mstatus_after & (1 << 7), (1 << 7));
        assert_eq!(mstatus_after & (3 << 11), 0);
    }

    #[test]
    fn test_exception_delegation() {
        let mut csr = CsrFile::default();
        let fault_pc = 0x8000_3000;
        let cause = Exception::Breakpoint;

        csr.write(0x302, 1 << 3, PrivilegeMode::Machine).unwrap();
        
        let s_handler = 0x8000_4000;
        csr.write(0x105, s_handler, PrivilegeMode::Supervisor).unwrap();

        let (next_mode, next_pc) = csr.trap_entry(fault_pc, cause, PrivilegeMode::User);

        assert_eq!(next_mode, PrivilegeMode::Supervisor);
        assert_eq!(next_pc, s_handler);
        
        assert_eq!(csr.sepc, fault_pc);
        assert_eq!(csr.scause, u32::from(cause));

        assert_eq!(csr.mcause, 0); 
    }
}