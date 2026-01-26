use crate::bits_op::BitsOp;
use crate::error::DecodeError;
use super::instruction::*;
use super::opcode::OpCode;
use super::instruction::Instruction::*;

/// Turn 32 bits instruction to corresponding `Instruction` enum
/// May return `RiscVError` of `NotImplementedOpCode`
///
/// ## Example
/// ```rust
/// # use riscv_decoder::prelude::*;
/// # use decoder::decode;
/// // add x5, x6, x7
/// let raw: u32 = 0x007302b3;
/// let execpt = Instruction::Base(Rv32iOp::Add, InstructionData{rd: 5, rs1: 6, rs2: 7, imm: 0});
///
/// assert_eq!(decode(raw), Ok(execpt));
/// assert_eq!(decode(0x01), Err(DecodeError::UnknownOpcode(0x01)));
/// ```
pub fn decode(raw: u32) -> Result<Instruction, DecodeError> {
    let opcode = raw.get_bits(0, 7) as u8;
    let rd = raw.get_bits(7, 5) as u8;
    let rs1 = raw.get_bits(15, 5) as u8;
    let rs2 = raw.get_bits(20, 5) as u8;
    let funct3 = raw.get_bits(12, 3) as u8;
    let funct7 = raw.get_bits(25, 7) as u8;

    match OpCode::try_from(opcode)? {
        // imm [11:0] | rs1 [4:0] | funct3 [2:0] | rd [4:0] | opcode [6:0]
        itype @ (OpCode::ItypeAr | OpCode::ItypeLoad | OpCode::ItypeJump | OpCode::ItypeFence | OpCode::ItypeSystem) => {
            let imm = raw.get_bits_signed(20, 12);
            
            if let Some(op) = Rv32iOp::decode_itype(itype, funct3, funct7, imm as u16) {
                let res = Base(op, InstructionData { rd, rs1, rs2, imm });
                return Ok(res);
            } 
            
            if itype == OpCode::ItypeSystem && let Some(op) = ZicsrOp::decode(funct3) {
                let res = Ziscr(op, InstructionData { rd, rs1, rs2, imm });
                return Ok(res);
            } 
            
            if itype == OpCode::ItypeFence && let Some(op) =  ZifenceiOp::decode(funct3) {
                let res = Zifencei(op, InstructionData { rd, rs1, rs2, imm });
                return Ok(res);
            } 
            
            if itype == OpCode::ItypeSystem && let Some(op) =  PrivilegeOp::decode(raw) {
                let res = Privileged(op);
                return Ok(res);
            } 
                
            Err(DecodeError::UnknownInstruction(itype, raw))
        },
        // funct7 [6:0] | rs2 [4:0] | rs1 [4:0] | funct3 [2:0]  | rd [4:0] | opcode [6:0]
        rtype @ OpCode::Rtype => {
            Ok(if let Some(op) = Rv32iOp::decode_rtype(funct3, funct7) {
                Base(op, InstructionData { rd, rs1, rs2, imm: 0 })
            } else if let Some(op) = MOp::decode(funct3, funct7) {
                M(op, InstructionData { rd, rs1, rs2, imm: 0 })
            } else {
                return  Err(DecodeError::UnknownInstruction(rtype, raw))
            })
        },
        // imm [11:5] | rs2 [4:0] | rs1 [4:0] | funct3 [2:0] | imm [4:0] | opcode [6:0]
        stype @ OpCode::Stype => {
            let imm = (raw.get_bits_signed(25, 7) << 5) | raw.get_bits(7, 5) as i32;
            if let Some(op) = Rv32iOp::decode_stype(funct3) {
                Ok(Base(op, InstructionData { rd, rs1, rs2, imm }))
            } else {
                Err(DecodeError::UnknownInstruction(stype, raw))
            }
        },
        // imm[12|10:5] | rs2 [4:0] | rs1 [4:0] | funct3 [2:0] | imm[4:1|11] | opcode [6:0]
        btype @ OpCode::Btype => {
            let imm = (raw.get_bits_signed(31, 1) << 12)
                | (raw.get_bits(7, 1) << 11) as i32
                | (raw.get_bits(25, 6) << 5) as i32
                | (raw.get_bits(8, 4) << 1) as i32;
            if let Some(op) = Rv32iOp::decode_btype(funct3) {
                Ok(Base(op, InstructionData { rd, rs1, rs2, imm }))
            } else {
                Err(DecodeError::UnknownInstruction(btype, raw))
            }
        },
        // imm[20|10:1|11|19:12] | rd[4:0] | opcode[6:0]
        jtype @ OpCode::Jtype => {
            let imm = (raw.get_bits_signed(31, 1) << 12)
                | (raw.get_bits(12, 8) << 12) as i32
                | (raw.get_bits(20, 1) << 11) as i32
                | (raw.get_bits(21, 10) << 1) as i32;
            if let Some(op) = Rv32iOp::decode_jtype() {
                Ok(Base(op, InstructionData { rd, rs1, rs2, imm }))
            } else {
                Err(DecodeError::UnknownInstruction(jtype, raw))
            }
        },
        // imm[31:12] | rd[4:0] | opcode[6:0]
        utype @ (OpCode::UtypeAuipc | OpCode::UtypeLui) => {
            let imm = raw.get_bits_signed(12, 20) << 12;

            if let Some(op) = Rv32iOp::decode_utype(utype) {
                Ok(Base(op, InstructionData { rd, rs1, rs2, imm }))
            } else {
                Err(DecodeError::UnknownInstruction(utype, raw))
            }
        },
    }
}

#[cfg(test)]
mod tests {
    mod rv32i {
        use crate::instruction::{Instruction, InstructionData};
        use crate::instruction::Rv32iOp;
        use crate::decoder::decode;

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
    }

    mod m {
        use crate::decoder::decode;
        use crate::instruction::{Instruction, InstructionData};
        use crate::instruction::MOp;

        fn build_m_data(op: MOp, rd: u8, rs1: u8, rs2: u8, imm: i32) -> Instruction {
            let data = InstructionData { rd, rs1, rs2, imm };
            Instruction::M(op, data)
        }

        #[test]
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
    }

    mod zicsr {
        use crate::decoder::decode;
        use crate::instruction::{Instruction, InstructionData};
        use crate::instruction::ZicsrOp;
        
        fn build_zicsr_data(op: ZicsrOp, rd: u8, rs1: u8, rs2: u8, imm: i32) -> Instruction {
            let data = InstructionData { rd, rs1, rs2, imm };
            Instruction::Ziscr(op, data)
        }
        
        #[test]
        fn  test_zicsr() {
            // csrrw x0, mstatus, x5
            let ins1 = 0x30029073;
            let expect1 = build_zicsr_data(ZicsrOp::Csrrw, 0, 5, 0, 0x300);
            // csrrci x0, mepc, 6
            let ins2 = 0x34137073;
            let expect2 = build_zicsr_data(ZicsrOp::Csrrci, 0, 6, 1, 0x341);

            assert_eq!(decode(ins1), Ok(expect1));
            assert_eq!(decode(ins2), Ok(expect2));

            if let Instruction::Ziscr(op1, _) = decode(ins1).unwrap() {
                assert!(op1.is_rw());
                assert!(!op1.is_imm());
            } else {
                unreachable!("Should be Base of Instruction");
            }
            
            if let Instruction::Ziscr(op2, _) = decode(ins2).unwrap() {
                assert!(op2.is_rc());
                 assert!(op2.is_imm());
            } else {
                unreachable!("Should be Base of Instruction");
            }
        }
    }

    mod zifencei {
        use crate::decoder::decode;
        use crate::instruction::{Instruction, InstructionData};
        use crate::instruction::ZifenceiOp;
              
        #[test]
        fn  test_zifencei() {
            // fence.i
            let ins = 0x0000100f;
            let expect = 
                Instruction::Zifencei(ZifenceiOp::FenceI,  
                    InstructionData { rd: 0, rs1: 0, rs2: 0, imm: 0 });

            assert_eq!(decode(ins), Ok(expect));
        }
    }

    mod privileged {
        use crate::decoder::decode;
        use crate::instruction::{Instruction, PrivilegeOp};

        #[test]
        fn  test_privileged() {
            // sret
            let ins1 = 0x10200073;
            let expect1 = Instruction::Privileged(PrivilegeOp::Sret);
            // mret
            let ins2 = 0x30200073;
            let expect2 = Instruction::Privileged(PrivilegeOp::Mret);

            assert_eq!(decode(ins1), Ok(expect1));
            assert_eq!(decode(ins2), Ok(expect2));
        }
    }
    
    use crate::decoder::decode;
    use crate::error::DecodeError;
    use crate::opcode::OpCode;

    #[test]
    fn  test_illegal() {
        let err_ins1 = 0xffffffff;
        let expect_err1 = Err(DecodeError::UnknownOpcode(0x7f));
        let err_ins2 = 0x00200073;
        let expect_err2 = Err(DecodeError::UnknownInstruction(OpCode::ItypeSystem, 0x00200073));

        assert_eq!(decode(err_ins1), expect_err1);     
        assert_eq!(decode(err_ins2), expect_err2);
    }
}
