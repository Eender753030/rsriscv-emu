#![cfg(any(feature = "zicsr", feature = "zifencei"))]

use riscv_decoder::decoder::decode;
use riscv_decoder::instruction::{Instruction, InstructionData};
#[cfg(feature = "s")]
use riscv_decoder::instruction::PrivilegeOp;
#[cfg(feature = "zicsr")]
use riscv_decoder::instruction::ZicsrOp; 
#[cfg(feature = "zifencei")]
use riscv_decoder::instruction::ZifenceiOp;


#[cfg(feature = "zicsr")]
fn build_zicsr_data(op: ZicsrOp, rd: u8, rs1: u8, rs2: u8, imm: i32, raw: u32) -> Instruction {
    let data = InstructionData { rd, rs1, rs2, imm };
    Instruction::Zicsr(op, data, raw)
}

#[test]
#[cfg(feature = "zicsr")]
fn  test_zicsr() {
    // csrrw x0, mstatus, x5
    let ins1 = 0x30029073;
    let expect1 = build_zicsr_data(ZicsrOp::Csrrw, 0, 5, 0, 0x300, ins1);
    // csrrci x0, mepc, 6
    let ins2 = 0x34137073;
    let expect2 = build_zicsr_data(ZicsrOp::Csrrci, 0, 6, 1, 0x341, ins2);

    assert_eq!(decode(ins1), Ok(expect1));
    assert_eq!(decode(ins2), Ok(expect2));

    if let Instruction::Zicsr(op1, _, _) = decode(ins1).unwrap() {
        assert!(op1.is_rw());
        assert!(!op1.is_imm());
    } else {
        unreachable!("Should be Base of Instruction");
    }
    
    if let Instruction::Zicsr(op2, _, _) = decode(ins2).unwrap() {
        assert!(op2.is_rc());
            assert!(op2.is_imm());
    } else {
        unreachable!("Should be Base of Instruction");
    }
}

#[test]
#[cfg(feature = "zifencei")]
fn  test_zifencei() {
    // fence.i
    let ins = 0x0000100f;
    let expect = 
        Instruction::Zifencei(ZifenceiOp::FenceI,  
            InstructionData { rd: 0, rs1: 0, rs2: 0, imm: 0 });

    assert_eq!(decode(ins), Ok(expect));
}


#[test]
#[cfg(feature = "s")]
fn  test_privileged() {
    // sret
    let ins1 = 0x10200073;

    // mret
    let ins2 = 0x30200073;

    // sfence.vma x2, x1
    let ins3 = 0x12110073;
    let expect3 = Instruction::Privileged(PrivilegeOp::SfenceVma(ins3),
        InstructionData { rd: 0, rs1: 2, rs2: 1, imm: 0 });

    assert!(matches!(decode(ins1), Ok(Instruction::Privileged(PrivilegeOp::Sret, _))));
    assert!(matches!(decode(ins2), Ok(Instruction::Privileged(PrivilegeOp::Mret, _))));
    assert_eq!(decode(ins3), Ok(expect3));
}