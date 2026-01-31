use COp::*;
use CFormat::*;

use crate::prelude::Rv32iOp;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum COp {
    Addi4spn, Lw, Sw,
    Nop, Addi, Jal, Li, Addi16sp, Lui, Srli,
    Srai, Andi, Sub, Xor, Or, And, J, Beqz, Bnez,
    Slli, Lwsp, Jr, Mv, Ebreak, Jalr, Add, Swsp,
    // Some F and D extension not implement yet
}

pub(crate) enum CFormat {
    Cr,
    Ci,
    Css,
    Ciw,
    Cl,
    Cs,
    Ca,
    Cb,
    Cj,
}

impl COp {
    pub(crate) fn decode_q0(raw: u16, funct3: u8) -> Option<(COp, CFormat)> {
        Some(match funct3 {
            0x0 if raw & 0x1fe != 0 => (Addi4spn, Ciw),
            0x2 => (Lw, Cl),
            0x6 => (Sw, Cs),
            _ => return None,
        })
    }

    pub(crate) fn decode_q1(raw: u16, funct3: u8, rd: u8) -> Option<(COp, CFormat)> {
        if raw & 0xef83 == 0x0001 {
            return Some((Nop, Ci));
        }
        
        Some(match funct3 {
            0x0 if rd != 0 => (Addi, Ci),
            0x1 => (Jal, Cj),
            0x2 => (Li, Ci),
            0x3 => match rd {
                0x02 if raw & 0x1000 != 0 || raw & 0x007c != 0 => (Addi16sp, Ci),
                _  if raw & 0x1000 != 0 || raw & 0x007c != 0 => (Lui, Ci),
                _    => return None,
            }
            0x4 => match (raw & 0xc00) >> 10 {
                0x0 => (Srli, Cb),
                0x1 => (Srai, Cb),
                0x2 => (Andi, Cb),
                _   => if (raw & 0x1000) != 0 {
                    return None;
                } else {
                    match (raw & 0x60) >> 5 {
                        0x0 => (Sub, Ca),
                        0x1 => (Xor, Ca),
                        0x2 => (Or, Ca),
                        0x3 => (And, Ca),
                        _   => unreachable!("Here include all situations"),
                    }
                },
            }
            0x5 => (J, Cj),
            0x6 => (Beqz, Cb),
            0x7 => (Bnez, Cb),
            _   => return None
        })
    }

    pub(crate) fn decode_q2(raw: u16, funct3: u8, rd: u8, rs2: u8) -> Option<(COp, CFormat)> {
        if raw == 0x9002 {
            return Some((Ebreak, Cr));
        }
        
        Some(match funct3 {
            0x0 => (Slli, Ci),
            0x2 if rd != 0 => (Lwsp, Ci),
            0x4 => match (raw & 0x1000) >> 12 {
                0x0 if rd != 0 && rs2 == 0 => (Jr, Cr),
                0x0 if rs2 != 0 => (Mv, Cr),
                0x1 if rd != 0 && rs2 == 0 => (Jalr, Cr),
                0x1 if rs2 != 0 => (Add, Cr),
                _   => return None,
            }
            0x6 => (Swsp, Css),
            _   => return None,
        })
    }

    pub fn is_uimm(&self) -> bool {
        matches!(self, 
            Addi4spn | Lw | Sw | Srli | Srai | Slli | Lwsp | Swsp )
    }

    pub(crate) fn into_base(self) -> Rv32iOp {
        match self {
            Lwsp | Lw      => Rv32iOp::Lw,
            Swsp | Sw      => Rv32iOp::Sw,
            Addi4spn       => Rv32iOp::Addi,
            J | Jal        => Rv32iOp::Jal,
            Jr | Jalr      => Rv32iOp::Jalr,
            Beqz           => Rv32iOp::Beq,
            Bnez           => Rv32iOp::Bne,
            Li | Addi |
            Nop | Addi16sp => Rv32iOp::Addi,
            Lui            => Rv32iOp::Lui,
            Mv | Add       => Rv32iOp::Add,
            Sub            => Rv32iOp::Sub,
            And            => Rv32iOp::And,
            Or             => Rv32iOp::Or,
            Xor            => Rv32iOp::Xor,
            Slli           => Rv32iOp::Slli,
            Srli           => Rv32iOp::Srli,
            Srai           => Rv32iOp::Srai,
            Andi           => Rv32iOp::Andi,
            Ebreak         => Rv32iOp::Ebreak,      
        }
    }
}

impl std::fmt::Display for COp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad( 
            match self {
                Addi4spn => "c.addi4spn",
                Lw       => "c.lw",
                Sw       => "c.sw",
                Nop      => "c.nop",
                Addi     => "c.addi",
                Jal      => "c.jal",
                Li       => "c.li",
                Addi16sp => "c.addi16sp",
                Lui      => "c.lui",
                Srli     => "c.srli",
                Srai     => "c.srai",
                Andi     => "c.andi",
                Sub      => "c.sub",
                Xor      => "c.xor",
                Or       => "c.or",
                And      => "c.and",
                J        => "c.j",
                Beqz     => "c.beqz",
                Bnez     => "c.bnez", 
                Slli     => "c.slli",  
                Lwsp     => "c.lwsp",
                Jr       => "c.jr", 
                Mv       => "c.mv", 
                Ebreak   => "c.ebreak", 
                Jalr     => "c.jalr", 
                Add      => "c.add",
                Swsp     => "c.swsp",
            }
        )
    }
}