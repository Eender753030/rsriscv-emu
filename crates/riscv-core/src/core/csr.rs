use modular_bitfield::prelude::*;
use crate::exception::Exception;

#[bitfield]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Mstatus {
    #[skip] __: B2,
    mie: B1,
    #[skip] __: B3,
    mpie: B1,
    #[skip] __: B25,
}

impl Mstatus {
    fn reset(&mut self) {
        self.set_mie(0);
        self.set_mpie(0);
    }
}

impl From<Mstatus> for u32 {
    fn from(value: Mstatus) -> Self {
        u32::from_le_bytes(value.into_bytes())
    }
}

impl Default for Mstatus {
    fn default() -> Self {
        Mstatus::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CsrFile {
    mstatus: Mstatus,
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
    pub fn read(&mut self, addr: u16) -> Result<u32, Exception> {
        Ok(match CsrAddr::try_from(addr)? {
            CsrAddr::Ustatus => 0,
            CsrAddr::Satp => 0,
            CsrAddr::Mstatus => self.mstatus.into(),
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
                self.mstatus.set_mie(mie);
                self.mstatus.set_mpie(mpie);
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

    pub fn trap_entry(&mut self, curr_pc: u32, except_code: Exception) -> u32 {
        self.mepc = curr_pc;
        self.mcause = except_code.into();
        self.mstatus.set_mpie(self.mstatus.mie());
        self.mstatus.set_mie(0);
        self.mtvec
    } 

    pub fn trap_ret(&mut self) -> u32 {
        self.mstatus.set_mie(self.mstatus.mpie());
        self.mstatus.set_mpie(1);
        self.mepc
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
            ("stap".to_string(), 0),
            ("mstatus".to_string(), self.mstatus.into()),
            ("medeleg".to_string(), 0),
            ("mideleg".to_string(), 0),
            ("mie".to_string(), 0),
            ("mtvec".to_string(), self.mtvec),
            ("mscratch".to_string(), self.mscratch),
            ("mepc".to_string(), self.mepc),
            ("mcause".to_string(), self.mcause),
            ("pmpcfg0".to_string(), 0),
            ("pmpaddr0".to_string(), 0),
            ("mnscratch".to_string(), self.mnscratch),
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