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
        itype @ (OpCode::ItypeAr | OpCode::ItypeLoad | OpCode::ItypeJump | OpCode::ItypeFence) => {
            let imm = raw.get_bits_signed(20, 12);
            
            if let Some(op) = Rv32iOp::decode_itype(itype, funct3, funct7) {
                let res = Base(op, InstructionData { rd, rs1, rs2, imm });
                return Ok(res);
            } 
            
            #[cfg(feature = "zifencei")]
            if itype == OpCode::ItypeFence && let Some(op) =  ZifenceiOp::decode(funct3) {
                let res = Zifencei(op, InstructionData { rd, rs1, rs2, imm });
                return Ok(res);
            } 
                
            Err(DecodeError::UnknownInstruction(itype, raw))
        },
        // funct7 [6:0] | rs2 [4:0] | rs1 [4:0] | funct3 [2:0]  | rd [4:0] | opcode [6:0]
        rtype @ OpCode::Rtype => {
            if let Some(op) = Rv32iOp::decode_rtype(funct3, funct7) {
                let res = Base(op, InstructionData { rd, rs1, rs2, imm: 0 });
                return Ok(res);
            } 
            
            #[cfg(feature = "m")]
            if let Some(op) = MOp::decode(funct3, funct7) {
                let res = M(op, InstructionData { rd, rs1, rs2, imm: 0 });
                return Ok(res);
            } 

            Err(DecodeError::UnknownInstruction(rtype, raw))
        },
        // imm [11:5] | rs2 [4:0] | rs1 [4:0] | funct3 [2:0] | imm [4:0] | opcode [6:0]
        stype @ OpCode::Stype => {
            let imm = (raw.get_bits_signed(25, 7) << 5) | raw.get_bits(7, 5) as i32;
            
            if let Some(op) = Rv32iOp::decode_stype(funct3) {
                let res = Ok(Base(op, InstructionData { rd, rs1, rs2, imm }));
                return res;
            } 
            
            Err(DecodeError::UnknownInstruction(stype, raw)) 
        },
        // imm[12|10:5] | rs2 [4:0] | rs1 [4:0] | funct3 [2:0] | imm[4:1|11] | opcode [6:0]
        btype @ OpCode::Btype => {
            let imm = (raw.get_bits_signed(31, 1) << 12)
                | (raw.get_bits(7, 1) << 11) as i32
                | (raw.get_bits(25, 6) << 5) as i32
                | (raw.get_bits(8, 4) << 1) as i32;
            
            if let Some(op) = Rv32iOp::decode_btype(funct3) {
                let res = Base(op, InstructionData { rd, rs1, rs2, imm });
                return Ok(res);
            } 

            Err(DecodeError::UnknownInstruction(btype, raw))
        },
        // imm[20|10:1|11|19:12] | rd[4:0] | opcode[6:0]
        jtype @ OpCode::Jtype => {
            let imm = (raw.get_bits_signed(31, 1) << 12)
                | (raw.get_bits(12, 8) << 12) as i32
                | (raw.get_bits(20, 1) << 11) as i32
                | (raw.get_bits(21, 10) << 1) as i32;
            
            if let Some(op) = Rv32iOp::decode_jtype() {
                let res = Base(op, InstructionData { rd, rs1, rs2, imm });
                return Ok(res);
            } 

            Err(DecodeError::UnknownInstruction(jtype, raw))
        },
        // imm[31:12] | rd[4:0] | opcode[6:0]
        utype @ (OpCode::UtypeAuipc | OpCode::UtypeLui) => {
            let imm = raw.get_bits_signed(12, 20) << 12;

            if let Some(op) = Rv32iOp::decode_utype(utype) {
                let res = Base(op, InstructionData { rd, rs1, rs2, imm });
                return Ok(res);
            } 

            Err(DecodeError::UnknownInstruction(utype, raw))  
        },
        #[cfg(feature = "a")]
        atomic @ OpCode::Amo => {
            let funct5 = raw.get_bits(27, 5) as u8;

            if let Some(op) = AOp::decode(funct5, funct3, rs2) {
                let rl = raw.get_bits(25, 1) as u8;
                let aq = raw.get_bits(26, 1) as u8;
                let res = A(op, AmoInsData { rd, rs1, rs2, rl, aq });
                return Ok(res);
            }

            Err(DecodeError::UnknownInstruction(atomic, raw))
        },
        system @ OpCode::System => {
            let imm = raw.get_bits(20, 12) as i32;

            if let Some(op) = Rv32iOp::decode_system(funct3, imm as u16) {
                let res = Base(op, InstructionData { rd, rs1, rs2, imm });
                return Ok(res);
            } 
            
            #[cfg(feature = "zicsr")]
            if let Some(op) = ZicsrOp::decode(funct3) {
                let res = Zicsr(op, InstructionData { rd, rs1, rs2, imm }, raw);
                return Ok(res);
            } 
            
            #[cfg(feature = "zicsr")]
            if let Some(op) =  PrivilegeOp::decode(raw, funct3, funct7, rd) {
                let res = Privileged(op, InstructionData { rd, rs1, rs2, imm: 0 });
                return Ok(res);
            } 
            
            Err(DecodeError::UnknownInstruction(system, raw))
        },
    }
}

#[cfg(feature = "c")]
pub fn decompress(raw: u16) -> Result<Instruction, DecodeError> {
    let opcode = raw.get_bits(0, 2) as u8;
    let funct3 = raw.get_bits(13, 3) as u8;
    let mut rs2 = raw.get_bits(2, 5) as u8;
    let rd_cr_ci = raw.get_bits(7, 5) as u8;

    if let Some((ins, cformat)) = match opcode {
        0b00 => COp::decode_q0(raw, funct3),
        0b01 => COp::decode_q1(raw, funct3, rd_cr_ci),
        0b10 => COp::decode_q2(raw, funct3, rd_cr_ci, rs2),
        _    => return Err(DecodeError::NotCompressInstruction),
    } {
        let rd;
        let mut rs1;
        let imm;

        match cformat {
            CFormat::Cr => {
                rd = if ins == COp::Jr { 0 } 
                    else if ins == COp::Jalr { 1 }
                    else { rd_cr_ci };
                rs1 = if ins == COp::Mv { 0 } else { rd_cr_ci };
                imm = 0;
            },
            CFormat::Ci => {
                rd = match ins {
                    COp::Nop => 0,
                    COp::Addi16sp => 2,
                    _ => rd_cr_ci,
                };
                rs1 = match ins {
                    COp::Li | COp::Nop => 0,
                    COp::Addi16sp => 2,
                    _ => rd_cr_ci,
                }; 
                rs2 = 0;
                imm = match ins {
                    COp::Addi16sp => {
                        rs1 = 2;
                        (raw.get_bits_signed(12, 1) << 9) as i32 |
                        (raw.get_bits(3, 2) << 7 |
                        raw.get_bits(5, 1) << 6 |
                        raw.get_bits(2, 1) << 5 |
                        raw.get_bits(6, 1) << 4) as i32
                    },
                    COp::Lui => {
                        (raw.get_bits_signed(12, 1) as i32) << 17 |
                        (raw.get_bits(2, 5) as i32) << 12
                    },
                    COp::Lwsp => {
                        rs1 = 2;
                        (raw.get_bits(2, 2)  << 6 |
                        raw.get_bits(12, 1) << 5 |
                        raw.get_bits(4, 3)  << 2) as i32
                    },
                    _ => (if ins.is_uimm() {
                        raw.get_bits(12, 1) as i32
                    } else {
                        raw.get_bits_signed(12, 1) as i32
                    } << 5 | raw.get_bits(2, 5) as i32),
                };
            },
            CFormat::Css => {
                rd = 0;
                rs1 = 2;
                imm = (raw.get_bits(7, 2) << 6 | raw.get_bits(9, 3) << 2) as i32;
            },
            CFormat::Ciw => {
                rd = raw.get_bits(2, 3) as u8 + 8;
                rs1 = 2;
                rs2 = 0;
                imm = (raw.get_bits(7, 4) << 6  | raw.get_bits(11, 2) << 4 |
                    raw.get_bits(5, 1) << 3  | raw.get_bits(6, 1) << 2) as i32;
            },
            CFormat::Cl | CFormat::Cs => {
                rd = raw.get_bits(2, 3) as u8 + 8;
                rs1 = raw.get_bits(7, 3) as u8 + 8;
                rs2 = rd;
                imm = (raw.get_bits(5, 1) << 6  | raw.get_bits(10, 3) << 3 |
                    raw.get_bits(6, 1) << 2) as i32;
            },
            CFormat::Ca => {
                rd = raw.get_bits(7, 3) as u8 + 8;
                rs1 = rd;
                rs2 = raw.get_bits(2, 3) as u8 + 8;
                imm = 0;
            },
            CFormat::Cb => {
                rd = raw.get_bits(7, 3) as u8 + 8;
                rs1 = rd;
                rs2 = 0;
                imm = match ins {
                    COp::Beqz | COp::Bnez => {
                        ((raw.get_bits_signed(12, 1) << 8) as i32) | (raw.get_bits(5, 2) << 6 |
                        raw.get_bits(2, 1) << 5 | raw.get_bits(10, 2) << 3 | raw.get_bits(3, 2) << 1) as i32
                    },
                    _ => (if ins.is_uimm() {
                        raw.get_bits(12, 1) as i32
                    } else {
                        raw.get_bits_signed(12, 1) as i32
                    } << 5 | raw.get_bits(2, 5) as i32) ,
                };
            },
            CFormat::Cj => {
                rd = if ins == COp::J { 0 } else { 1 };
                rs1 = 0;
                rs2 = 0;
                imm = (raw.get_bits_signed(12, 1) << 11) as i32 | (raw.get_bits(8, 1) << 10 |
                raw.get_bits(9, 2) << 8 | raw.get_bits(6, 1) << 7 | raw.get_bits(7, 1) << 6 |
                raw.get_bits(2, 1) << 5 | raw.get_bits(11, 1) << 4 | raw.get_bits(3, 3) << 1) as i32;
            }
        }
        let res = Instruction::Base(ins.into_base(), InstructionData { rd, rs1, rs2, imm });
        return Ok(res);
    }

    Err(DecodeError::UnknownCompressInstruction(opcode, raw)) 
}