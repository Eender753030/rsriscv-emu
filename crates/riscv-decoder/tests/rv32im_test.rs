use riscv_decoder::DecodeError;
use riscv_decoder::decoder::decode;
use riscv_decoder::instruction::{Instruction, InstructionData, Rv32iOp};
#[cfg(feature = "m")]
use riscv_decoder::instruction::MOp;

fn build_base_data(op: Rv32iOp, rd: u8, rs1: u8, rs2: u8, imm: i32) -> Instruction {
    let data = InstructionData { rd, rs1, rs2, imm };
    Instruction::Base(op, data)
}

#[test]
fn test_itype_arithmetic() {
    // addi x10, x0, -2048
    let ins1 = 0x80000513;
    let expect1 = build_base_data(Rv32iOp::Addi, 10, 0, 0, -2048);
    
    // slti x23, x24, 31
    let ins2 = 0x01fc2b93;
    let expect2 = build_base_data(Rv32iOp::Slti, 23, 24, 31, 31);

    assert_eq!(decode(ins1), Ok(expect1));
    assert_eq!(decode(ins2), Ok(expect2));

    if let Instruction::Base(op1, _) = decode(ins1).unwrap() {
        assert!(op1.is_itype_ar());
    } else {
        unreachable!("Should be Base of Instruction");
    }
    
    if let Instruction::Base(op2, _) = decode(ins2).unwrap() {
        assert!(op2.is_itype_ar());
    } else {
        unreachable!("Should be Base of Instruction");
    }
}

#[test]
fn test_itype_load() {
    // lw x5, 12(x7)
    let ins1 = 0x00c3a283;
    let expect1 = build_base_data(Rv32iOp::Lw, 5, 7, 12, 12);
    // lbu x19, 0(x11)
    let ins2 = 0x0005c983;
    let expect2 = build_base_data(Rv32iOp::Lbu, 19, 11, 0, 0);

    assert_eq!(decode(ins1), Ok(expect1));
    assert_eq!(decode(ins2), Ok(expect2));

    if let Instruction::Base(op1, _) = decode(ins1).unwrap() {
        assert!(op1.is_itype_load());
    } else {
        unreachable!("Should be Base of Instruction");
    }
    
    if let Instruction::Base(op2, _) = decode(ins2).unwrap() {
        assert!(op2.is_itype_load());
    } else {
        unreachable!("Should be Base of Instruction");
    }
}

#[test]
fn  test_itype_jump() {
    // jalr x1, -442(x21)
    let ins = 0xe46a80e7;
    let expect = build_base_data(Rv32iOp::Jalr, 1, 21, 6, -442);

    assert_eq!(decode(ins), Ok(expect));

    if let Instruction::Base(op, _) = decode(ins).unwrap() {
        assert!(op.is_itype_jump());
    } else {
        unreachable!("Should be Base of Instruction");
    }
}

#[test]
fn  test_sys() {
    // ecall
    let ins1 = 0x00000073;
    let expect1 = build_base_data(Rv32iOp::Ecall, 0, 0, 0, 0);
    // ebreak
    let ins2 = 0x00100073;
    let expect2 = build_base_data(Rv32iOp::Ebreak, 0, 0, 1, 1);

    assert_eq!(decode(ins1), Ok(expect1));
    assert_eq!(decode(ins2), Ok(expect2));

    if let Instruction::Base(op1, _) = decode(ins1).unwrap() {
        assert!(op1.is_itype_system());
    } else {
        unreachable!("Should be Base of Instruction");
    }
    
    if let Instruction::Base(op2, _) = decode(ins2).unwrap() {
        assert!(op2.is_itype_system());
    } else {
        unreachable!("Should be Base of Instruction");
    }
}

#[test]
fn  test_rtype() {
    // sub x13, x15, x18
    let ins1 = 0x412786b3;
    let expect1 = build_base_data(Rv32iOp::Sub, 13, 15, 18, 0);
    // xor x20, x30, x26
    let ins2 = 0x01af4a33;
    let expect2 = build_base_data(Rv32iOp::Xor, 20, 30, 26, 0);

    assert_eq!(decode(ins1), Ok(expect1));
    assert_eq!(decode(ins2), Ok(expect2));

    if let Instruction::Base(op1, _) = decode(ins1).unwrap() {
        assert!(op1.is_rtype());
    } else {
        unreachable!("Should be Base of Instruction");
    }
    
    if let Instruction::Base(op2, _) = decode(ins2).unwrap() {
        assert!(op2.is_rtype());
    } else {
        unreachable!("Should be Base of Instruction");
    }
}

#[test]
fn  test_stype() {
    // sw x21, 123(x13)
    let ins1 = 0x0756ada3;
    let expect1 = build_base_data(Rv32iOp::Sw, 27, 13, 21, 123);
    // sh x11, -567(x22)
    let ins2 = 0xdcbb14a3;
    let expect2 = build_base_data(Rv32iOp::Sh, 9, 22, 11, -567);
    
    assert_eq!(decode(ins1), Ok(expect1));
    assert_eq!(decode(ins2), Ok(expect2));

    if let Instruction::Base(op1, _) = decode(ins1).unwrap() {
        assert!(op1.is_stype());
    } else {
        unreachable!("Should be Base of Instruction");
    }
    
    if let Instruction::Base(op2, _) = decode(ins2).unwrap() {
        assert!(op2.is_stype());
    } else {
        unreachable!("Should be Base of Instruction");
    }
}

#[test]
fn  test_jtype() {
    // jal x1, 32
    let ins = 0x020000ef;
    let expect = build_base_data(Rv32iOp::Jal, 1, 0, 0, 32);

    assert_eq!(decode(ins), Ok(expect));

    if let Instruction::Base(op, _) = decode(ins).unwrap() {
        assert!(op.is_jtype());
    } else {
        unreachable!("Should be Base of Instruction");
    }
}

#[test]
fn  test_utype() {
    // auipc x29, 0x100
    let ins1 = 0x00100e97;
    let expect1 = build_base_data(Rv32iOp::Auipc, 29, 0, 1, 0x100 << 12);
    // lui x19, 0x20000
    let ins2 = 0x200009b7;
    let expect2 = build_base_data(Rv32iOp::Lui, 19, 0, 0, 0x20000 << 12);

    assert_eq!(decode(ins1), Ok(expect1));
    assert_eq!(decode(ins2), Ok(expect2));

    if let Instruction::Base(op1, _) = decode(ins1).unwrap() {
        assert!(op1.is_utype());
    } else {
        unreachable!("Should be Base of Instruction");
    }
    
    if let Instruction::Base(op2, _) = decode(ins2).unwrap() {
        assert!(op2.is_utype());
    } else {
        unreachable!("Should be Base of Instruction");
    }
}

#[cfg(feature = "m")]
fn build_m_data(op: MOp, rd: u8, rs1: u8, rs2: u8, imm: i32) -> Instruction {
    let data = InstructionData { rd, rs1, rs2, imm };
    Instruction::M(op, data)
}

#[test]
#[cfg(feature = "m")]
fn  test_m() {
    // mulhsu x17, x16, x15
    let ins1 = 0x02f828b3;
    let expect1 = build_m_data(MOp::Mulhsu, 17, 16, 15, 0);
    // rem x23, x24, x25
    let ins2 = 0x039c6bb3;
    let expect2 = build_m_data(MOp::Rem, 23, 24, 25, 0);

    assert_eq!(decode(ins1), Ok(expect1));
    assert_eq!(decode(ins2), Ok(expect2));
}

#[test]
fn  test_illegal() {
    let err_ins1 = 0xffffffff;
    let expect_err1 = Err(DecodeError::UnknownOpcode(0x7f));
    let err_ins2 = 0x00200073;

    assert_eq!(decode(err_ins1), expect_err1);     
    assert!(matches!(decode(err_ins2), 
        Err(DecodeError::UnknownInstruction(_, 0x00200073))));
}