#[cfg(feature = "s")] use riscv_decoder::decoder::decode;
#[cfg(feature = "s")] use riscv_decoder::instruction::Instruction;
#[cfg(feature = "s")] use crate::Exception;
#[cfg(feature = "zicsr")] use crate::core::privilege::PrivilegeMode;
use crate::core::access::{Access, AccessType};
use crate::core::cpu::Cpu;
use crate::constance::DRAM_BASE_ADDR;


#[test]
fn test_cpu_initial_state() {
    let cpu = Cpu::default();
    assert_eq!(cpu.pc.get(), DRAM_BASE_ADDR, "PC should start at DRAM base");
    #[cfg(feature = "zicsr")]
    assert_eq!(cpu.mode, PrivilegeMode::Machine, "Should start in Machine Mode");
    assert_eq!(cpu.regs[1], 0);
}

#[test]
fn test_load_program_to_memory() {
    let mut cpu = Cpu::default();
    let code = vec![0xEF, 0xBE, 0xAD, 0xDE]; 
    
    cpu.load(DRAM_BASE_ADDR, &code).expect("Load failed");

    let access = Access::new(DRAM_BASE_ADDR, AccessType::Load);
    let val = cpu.bus.read_u32(access.bypass()).expect("Bus read failed");
    
    assert_eq!(val, 0xDEADBEEF, "Memory content mismatch");
}

#[test]
fn test_cycle_execution_addi() {
    // Fetch-Decode-Execute
    let mut cpu = Cpu::default();

    // addi x1, x0, 10
    let code = 0x00A00093u32.to_le_bytes();
    cpu.load(DRAM_BASE_ADDR, &code).unwrap();

    cpu.step().expect("Step failed");

    assert_eq!(cpu.pc.get(), DRAM_BASE_ADDR + 4, "PC did not advance");
    assert_eq!(cpu.regs[1], 10, "x1 register value incorrect");
}

#[test]
fn test_cycle_execution_add() {
    let mut cpu = Cpu::default();

    cpu.regs.write(1, 10); // x1 = 10
    cpu.regs.write(2, 20); // x2 = 20

    // add x3, x1, x2
    let code = 0x002081B3u32.to_le_bytes();
    cpu.load(DRAM_BASE_ADDR, &code).unwrap();

    cpu.step().unwrap();

    // x3 = 10 + 20 = 30
    assert_eq!(cpu.regs[3], 30);
}

#[test]
fn test_cycle_execution_bne_taken() {
    let mut cpu = Cpu::default();

    cpu.regs.write(1, 5);
    cpu.regs.write(2, 10);

    // bne x1, x2, 8
    let bne_code = 0x00209463u32.to_le_bytes();
    
    cpu.load(DRAM_BASE_ADDR, &bne_code).unwrap();

    cpu.step().unwrap();

    assert_eq!(cpu.pc.get(), DRAM_BASE_ADDR + 8, "Branch did not take");
}

#[test]
#[cfg(feature = "zicsr")]
fn test_exception_trap_handling() {
    let mut cpu = Cpu::default();

    // mtvec = 0x8000_0100
    let handler_base = DRAM_BASE_ADDR + 0x100;
    cpu.csrs.write(0x305, handler_base, PrivilegeMode::Machine, 0).unwrap();

    // Illegal: 0xFFFFFFFF
    let illegal_inst = 0xFFFFFFFFu32.to_le_bytes();
    cpu.load(DRAM_BASE_ADDR, &illegal_inst).unwrap();

    cpu.step().unwrap(); 

    assert_eq!(cpu.pc.get(), handler_base, "Did not trap to mtvec");

    let mcause = cpu.csrs.read(0x342, PrivilegeMode::Machine, 0).unwrap();
    assert_eq!(mcause, 2, "mcause wrong");

    let mepc = cpu.csrs.read(0x341, PrivilegeMode::Machine, 0).unwrap();
    assert_eq!(mepc, DRAM_BASE_ADDR, "mepc wrong");
}

#[test]
#[cfg(feature = "s")]
fn test_sfence_vma() {
    let mut cpu = Cpu::default();
    
    cpu.mode = PrivilegeMode::Supervisor;
    
    // sfence.vma x10, x11
    let raw = 0x12a58073;

    let rs1_idx = 10;
    let rs2_idx = 11;
    let vaddr_val = 0x8000_1000;
    let asid_val = 0x1;
    
    cpu.regs.write(rs1_idx, vaddr_val);
    cpu.regs.write(rs2_idx, asid_val);

    let ins = decode(raw).unwrap();
    
    let res = if let Instruction::Privileged(op, data) = ins {
        cpu.execute_privileged(op, data)
    } else {
        panic!("");
    };

    assert!(res.is_ok());
    let next_pc_manual = res.unwrap();
    assert_eq!(next_pc_manual, false);

    cpu.mode = PrivilegeMode::User;
    let res_err = if let Instruction::Privileged(op, data) = ins {
        cpu.execute_privileged(op, data)
    } else {
        panic!("");
    };
    match res_err {
        Err(Exception::IllegalInstruction(_)) => (),
        _ => panic!(""),
    }
}