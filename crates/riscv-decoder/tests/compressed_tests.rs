#![cfg(feature = "c")]
use riscv_decoder::decoder::decompress;
use riscv_decoder::prelude::{Rv32iOp, Instruction, InstructionData};


fn build_base_data(op: Rv32iOp, rd: u8, rs1: u8, rs2: u8, imm: i32) -> Instruction {
    let data = InstructionData { rd, rs1, rs2, imm };
    Instruction::Base(op, data)
}

#[test]
fn easy_tests() {
    let ins1 = 0x0001; // c.nop
    let expect1 = build_base_data(Rv32iOp::Addi, 0, 0, 0, 0);
    assert_eq!(decompress(ins1).unwrap(), expect1);

    let ins2 = 0x4501; // c.li x10, 0
    let expect2 = build_base_data(Rv32iOp::Addi, 10, 0, 0, 0); 
    assert_eq!(decompress(ins2).unwrap(), expect2);

    let ins3 = 0x8082; // c.jr x1
    let expect3 = build_base_data(Rv32iOp::Jalr, 0, 1, 0, 0); // rs2 don't care
    assert_eq!(decompress(ins3).unwrap(), expect3);

    let ins4 = 0x9002; // c.break
    let expect4 = Rv32iOp::Ebreak;
    if let Ok(Instruction::Base(op, _)) = decompress(ins4) {
        assert_eq!(op, expect4);
    } else {
        panic!("Decompress should pass")
    }

    let ins5 = 0x4188; // c.lw x10, 0(x11)
    let expect5 = build_base_data(Rv32iOp::Lw, 10, 11, 10, 0); // rs2 don't care
    assert_eq!(decompress(ins5).unwrap(), expect5);
}

#[test]
fn logic_tests() {
    let ins1 = 0x8d0d; // c.sub x10, x11
    let expect1 = build_base_data(Rv32iOp::Sub, 10, 10, 11, 0); // imm don't care
    assert_eq!(decompress(ins1).unwrap(), expect1);

    let ins2 = 0x8d6d; // c.and x10, x11
    let expect2 = build_base_data(Rv32iOp::And, 10, 10, 11, 0); // imm don't care
    assert_eq!(decompress(ins2).unwrap(), expect2);

    let ins3 = 0x15fd; // c.addi x11, -1
    let expect3 = build_base_data(Rv32iOp::Addi, 11, 11, 0, -1); // rs2 don't care
    assert_eq!(decompress(ins3).unwrap(), expect3);
    
    let ins4 = 0x0048; // c.addi4spn x10, 4
    let expect4 = build_base_data(Rv32iOp::Addi, 10, 2, 0, 4); // rs1 should be x2. rs2 don't care
    assert_eq!(decompress(ins4).unwrap(), expect4);
}

#[test]
fn trap_tests() {
    let ins1 = 0x717D; // c.addi16sp -16
    let expect1 = build_base_data(Rv32iOp::Addi, 2, 2, 0, -16); // rs2 don't care
    assert_eq!(decompress(ins1).unwrap(), expect1);

    let ins2 = 0x757D; // c.lui x10, 0xfffff (imm need to shift left 12)
    let expect2 = build_base_data(Rv32iOp::Lui, 10, 10, 0, -4096); // rs1 & rs2 don't care
    assert_eq!(decompress(ins2).unwrap(), expect2);

    let ins3 = 0x85AA; // c.mv x11, x10
    let expect3 = build_base_data(Rv32iOp::Add, 11, 0, 10, 0); // imm don't care
    assert_eq!(decompress(ins3).unwrap(), expect3);
    
    let ins4 = 0x95AA; // c.add x11, x10
    let expect4 = build_base_data(Rv32iOp::Add, 11, 11, 10, 0); // imm don't care
    assert_eq!(decompress(ins4).unwrap(), expect4);
}

#[test]
fn illegal_tests() {
    let ill1 = 0x0000;
    assert!(decompress(ill1).is_err());

    let ill2 = 0x6001;
    assert!(decompress(ill2).is_err());

    let ill3 = 0xA000;
    assert!(decompress(ill3).is_err());

    let not_compress = 0x0073;
    assert!(decompress(not_compress).is_err());
}