mod addr;
mod mstatus;
mod pmpcfg;
mod satp;

use crate::Exception;
use crate::core::{Access, Physical};
use crate::core::privilege::PrivilegeMode;

use addr::CsrAddr;
use mstatus::Mstatus;
use pmpcfg::Pmpcfg;
use satp::Satp;

pub(super) const PMPCFG_NUM: usize = 4;

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

    pmpcfg: [Pmpcfg; PMPCFG_NUM],
    pmpaddr: [u32; PMPCFG_NUM * 4],
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

                CsrAddr::Pmpcfg(num) => self.pmpcfg[num].into(),
                CsrAddr::Pmpaddr(num) => self.pmpaddr[num],
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

                CsrAddr::Pmpcfg(num) => self.pmpcfg[num] = data.into(),
                CsrAddr::Pmpaddr(num) => self.pmpaddr[num] = data, 
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
            _   => 0,
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

    pub fn check_satp(&self) -> Option<u32> {
        if self.satp.mode() > 0  {
            Some(self.satp.ppn())
        } else {
            None
        }
    }

    pub fn pmp_check(&self, access: Access<Physical>, size: usize, mode: PrivilegeMode) -> Result<(),Exception> {
        use pmpcfg::MatchingMode::*;
        let mut is_match = None;
        for i in 0..PMPCFG_NUM * 4 {
            if match self.pmpcfg[i / 4][i % 4].mode() {
                Off   => continue,
                Tor   => self.top_of_range(i, access.addr, size),
                Na4   => self.na4(i, access.addr, size),
                Napot => self.napot(i, access.addr, size),
            } {
                is_match = Some(i);
                break;
            }
        }
        match is_match {
            Some(idx) => {
                let pmpcfg = &self.pmpcfg[idx / 4][idx % 4];
                if pmpcfg.mode_check(mode) {
                    return Ok(());
                }
                if !pmpcfg.access_check(access) {
                    return Err(access.to_access_exception());
                }
                Ok(())
            },
            None => match mode {
                PrivilegeMode::Machine => Ok(()),
                _                      => Err(access.to_access_exception()),
            }
        }
    }

    fn top_of_range(&self, idx: usize, addr: u32, size: usize) -> bool {
        let lower = match idx.checked_sub(1) {
            Some(i) => self.pmpaddr[i] << 2,
            None      => 0,
        };
        let upper = self.pmpaddr[idx] << 2;

        lower <= addr && addr + (size as u32) < upper
    }

    fn na4(&self, idx: usize, addr: u32, size: usize) -> bool {
        let base = self.pmpaddr[idx] << 2;

        base <= addr && addr + (size as u32) < base + 4
    }

    fn napot(&self, idx: usize, addr: u32, size: usize) -> bool {
        let mask_bit = self.pmpaddr[idx].trailing_ones();
        let chunck_size = 1 << (3 + mask_bit);
        let base = (self.pmpaddr[idx] & !((1 << mask_bit) - 1)) << 2;

        base <= addr && (addr as usize) + size < (base as usize)  + chunck_size
    }

    pub fn reset(&mut self) {
        *self = Self::default()
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

    mod pmp {
        use crate::core::{Access, AccessType, CsrFile};
        use crate::core::privilege::PrivilegeMode;
        use crate::exception::Exception;

        fn set_pmp_entry(csr: &mut CsrFile, idx: usize, cfg: u8, addr: u32) {
            let shift = (idx % 4) * 8;
            
            let mut curr_cfg = csr.read(0x3a0, PrivilegeMode::Machine).unwrap();
            curr_cfg &= !(0xff << shift);

            curr_cfg |= (cfg as u32) << shift;
            csr.write(0x3a0, curr_cfg, PrivilegeMode::Machine).unwrap();

            let csr_addr = 0x3b0 + idx as u16;
            csr.write(csr_addr, addr, PrivilegeMode::Machine).unwrap();
        }

        #[test]
        fn test_default() {
            let csr = CsrFile::default();
            let access = Access::new(0x8000_0000, AccessType::Load);
            let mode = PrivilegeMode::Machine;

            assert!(csr.pmp_check(access, 4, mode).is_ok());

            let mode = PrivilegeMode::Supervisor;
            assert_eq!(csr.pmp_check(access, 4, mode),
                Err(Exception::LoadAccessFault(0x8000_0000)));
        }

        #[test]
        fn test_tor() {
            let mut csr = CsrFile::default();
            let addr = 0x8000_0000;

            let cfg = (1 << 3) | (1 << 0); // A = 01, R = 1
            set_pmp_entry(&mut csr, 0, cfg, addr + 1000 >> 2);

            let mut access = Access::new(addr, AccessType::Load);
            let mode = PrivilegeMode::Supervisor;
    
            assert!(csr.pmp_check(access, 4, mode).is_ok());

            access.kind = AccessType::Store;
            assert_eq!(csr.pmp_check(access, 4, mode),
                Err(Exception::StoreAccessFault(addr)));

            access.kind = AccessType::Fetch;
            assert_eq!(csr.pmp_check(access, 4, mode), 
                Err(Exception::InstructionAccessFault(addr)));
        }

        #[test]
        fn test_na4() {
            let mut csr = CsrFile::default();
            let addr = 0x8000_0000;

            let cfg = (1 << 4) | (3 << 1); // A = 10, W = 1, X = 1
            set_pmp_entry(&mut csr, 0, cfg, addr >> 2);

            let mut access = Access::new(addr, AccessType::Store);
            let mode = PrivilegeMode::Supervisor;
    
            assert!(csr.pmp_check(access, 2, mode).is_ok());

            access.kind = AccessType::Fetch;
            assert!(csr.pmp_check(access, 2, mode).is_ok());

            access.kind = AccessType::Load;
            assert_eq!(csr.pmp_check(access, 2, mode),
                Err(Exception::LoadAccessFault(addr)));
        }

        #[test]
        fn test_napot() {
            let mut csr = CsrFile::default();
            let addr = 0x8000_0000;

            let cfg = (3 << 3) | (3 << 0); // A = 11, R = 1, W = 1
            let pmpaddr = (0x8000_0000 >> 2) | 0x3FF;
            set_pmp_entry(&mut csr, 0, cfg, pmpaddr);

            let mut access = Access::new(addr, AccessType::Load);
            let mode = PrivilegeMode::Supervisor;
    
            assert!(csr.pmp_check(access, 2, mode).is_ok());

            access.kind = AccessType::Store;
            assert!(csr.pmp_check(access, 2, mode).is_ok());

            access.kind = AccessType::Fetch;
            assert_eq!(csr.pmp_check(access, 4, mode), 
                Err(Exception::InstructionAccessFault(addr)));
        }

        #[test]
        fn test_priority() {
            let mut csr = CsrFile::default();
            let mode = PrivilegeMode::Supervisor;
            // pmp0: A = 01
            set_pmp_entry(&mut csr, 0, 1 << 3, 0x8000_1000 >> 2);
            // pmp1: A = 01, R = 1, W = 1, X = 1
            set_pmp_entry(&mut csr, 1, (1 << 3) | (7 << 0), 0x8000_2000 >> 2);

            let access0 = Access::new(0x8000_0050, AccessType::Load);
            assert!(csr.pmp_check(access0, 4, mode).is_err());
            let access1 = Access::new(0x8000_1050, AccessType::Load); 
            assert!(csr.pmp_check(access1, 4, mode).is_ok());            
        }

        #[test] 
        fn test_lock() {
            let mut csr = CsrFile::default();

            let cfg_unlocked = (1 << 3) | (1 << 0) ; // A = 01, R = 1 
            set_pmp_entry(&mut csr, 0, cfg_unlocked, 0x8000_1000 >> 2);

            let access = Access::new(0x8000_0050, AccessType::Store);
            assert!(csr.pmp_check(access, 4, PrivilegeMode::Machine).is_ok());

            let cfg_locked = (1 << 7) | (1 << 3) | (1 << 0) ; // L = 1, A = 01, R = 1 
            set_pmp_entry(&mut csr, 0, cfg_locked, 0x8000_1000 >> 2);

            assert_eq!(csr.pmp_check(access, 4, PrivilegeMode::Machine), 
                Err(Exception::StoreAccessFault(0x8000_0050)));
        }
    }
    
}