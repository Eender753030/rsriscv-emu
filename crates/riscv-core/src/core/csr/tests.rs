use crate::core::CsrFile;
use crate::core::privilege::PrivilegeMode;
use crate::exception::Exception;

#[test]
fn test_csr_rw_permission() {
    let mut csr = CsrFile::default();
    let val = 0xDEAD_BEEF;

    assert!(csr.write(0x340, val, PrivilegeMode::Machine).is_ok());
    assert_eq!(csr.read(0x340, PrivilegeMode::Machine), Ok(val));

    #[cfg(feature = "s")]
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
    
    #[cfg(feature = "s")] {
        let sstatus = csr.read(0x100, PrivilegeMode::Supervisor).unwrap();
        assert_eq!(sstatus, 0);
    }
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
#[cfg(feature = "s")]
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
    use crate::core::CsrFile;
    use crate::core::access::{Access, AccessType};
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

        #[cfg(feature = "s")] {
        let mode = PrivilegeMode::Supervisor;
        assert_eq!(csr.pmp_check(access, 4, mode),
            Err(Exception::LoadAccessFault(0x8000_0000)));
        }
    }

    #[test]
    fn test_tor() {
        let mut csr = CsrFile::default();
        let addr = 0x8000_0000;

        let cfg = (1 << 3) | (1 << 0); // A = 01, R = 1
        set_pmp_entry(&mut csr, 0, cfg, addr + 1000 >> 2);

        let mut access = Access::new(addr, AccessType::Load);
        let mode = PrivilegeMode::User;

        assert!(csr.pmp_check(access, 4, mode).is_ok());

        access.kind = AccessType::Store;
        assert_eq!(csr.pmp_check(access, 4, mode),
            Err(Exception::StoreOrAmoAccessFault(addr)));

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
        let mode = PrivilegeMode::User;

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
        let mode = PrivilegeMode::User;

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
        let mode = PrivilegeMode::User;
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
            Err(Exception::StoreOrAmoAccessFault(0x8000_0050)));
    }
}