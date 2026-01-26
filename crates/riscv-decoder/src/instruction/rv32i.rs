use crate::opcode::OpCode;

use Rv32iOp::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rv32iOp {
    // Itype Ar
    Addi, Slli, Slti, Sltiu,
    Xori, Srli, Srai, Ori, Andi,
    // Itype Load
    Lb, Lh, Lw, 
    Lbu, Lhu,
    // Itype Jump
    Jalr,
    // Itype Fence
    Fence,
    // Itype System
    Ecall, Ebreak,
    // Rtype
    Add, Sub, Sll, Slt, Sltu,
    Xor, Srl, Sra, Or, And,
    // Btype
    Beq, Bne, Blt, Bge,
    Bltu, Bgeu,
    // Stype
    Sb, Sh, Sw,
    // Jtype
    Jal,
    // Utype
    Lui, Auipc,
}

impl Rv32iOp {
    pub(crate) fn decode_itype(opcode: OpCode, funct3: u8, funct7: u8, funct12: u16) -> Option<Rv32iOp> {
        Some(match opcode {
            OpCode::ItypeAr => match funct3 {
                0x0 => Addi,
                0x1 => Slli,
                0x2 => Slti,
                0x3 => Sltiu,
                0x4 => Xori,
                0x5 => match funct7 {
                    0x00 => Srli,
                    0x20 => Srai,
                    _    => return None,
                },
                0x6 => Ori,
                0x7 => Andi,
                _   => return None,
            },
            OpCode::ItypeLoad => match funct3 {
                0x0 => Lb,
                0x1 => Lh,
                0x2 => Lw,
                0x4 => Lbu,
                0x5 => Lhu,
                _   => return None,
            },
            OpCode::ItypeJump => match funct3 {
                0x0 => Jalr,
                _   => return None,
            },
            OpCode::ItypeFence => match funct3 {
                0x0 => Fence,
                _   => return None,
            },
            OpCode::ItypeSystem => match funct3 {
                0x0 => match funct12 {
                    0x000 => Ecall,
                    0x001 => Ebreak,
                    _     => return None,
                },
                _ => return None,
            },
            _ => return None,
        })
    }

    pub(crate) fn decode_rtype(funct3: u8, funct7: u8) -> Option<Rv32iOp> {
        Some(match funct7 {
            0x00 => match funct3 {
                0x0 => Add,
                0x1 => Sll,
                0x2 => Slt,
                0x3 => Sltu,
                0x4 => Xor,
                0x5 => Srl,
                0x6 => Or,
                0x7 => And,
                _   => return None,
            }
            0x20 => match funct3 {
                0x0 => Sub,
                0x5 => Sra,
                _   => return None
            },
            _ => return None,
        })
        
    }

    pub(crate) fn decode_stype(funct3: u8) -> Option<Rv32iOp> {
        Some(match funct3 {
            0x0 => Sb,
            0x1 => Sh,
            0x2 => Sw,
            _   => return None,
        })
    }

    pub(crate) fn decode_btype(funct3: u8) -> Option<Rv32iOp> {
        Some(match funct3 {
            0x0 => Beq,
            0x1 => Bne,
            0x4 => Blt,
            0x5 => Bge,
            0x6 => Bltu,
            0x7 => Bgeu,
            _   => return None,
        })
    }

    pub(crate) fn decode_jtype() -> Option<Rv32iOp> {
        Some(Jal)
    }

    pub(crate) fn decode_utype(opcode: OpCode) -> Option<Rv32iOp> {
        Some(match opcode {
            OpCode::UtypeAuipc => Auipc,
            OpCode::UtypeLui   => Lui,
            _                  => return None,
        })
    }

    pub fn is_itype_ar(&self) -> bool {
        matches!(self, 
                Addi | Slli  |
                Slti | Sltiu |
                Xori | Srli  |
                Srai | Ori   |
                Andi
        )
    }

    pub fn is_itype_load(&self) -> bool {
        matches!(self, 
            Lb  | Lh | Lw |
            Lbu | Lhu
        )
    }

    pub fn is_itype_jump(&self) -> bool {
        self == &Jalr
    }

    pub fn is_itype_system(&self) -> bool {
        matches!(self, Ecall | Ebreak)

    }
    pub fn is_itype_fence(&self) -> bool {
        self == &Fence
    }

    pub fn is_rtype(&self) -> bool {
        matches!(self, 
            Add  | Sub |
            Sll  | Slt |
            Sltu | Xor |
            Srl  | Sra |
            Or   | And
        )
    }

    pub fn is_stype(&self) -> bool {
        matches!(self, Sb | Sh | Sw)
    }

    pub fn is_btype(&self) -> bool {
        matches!(self, 
            Beq  | Bne | 
            Blt  | Bge |
            Bltu | Bgeu
        )
    }

    pub fn is_jtype(&self) -> bool {
        self == &Jal
    }

    pub fn is_utype(&self) -> bool {
        matches!(self, Lui | Auipc)
    }
}

impl std::fmt::Display for Rv32iOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let op_str = match self {
            Addi  => "addi",  Slli => "slli",  Slti => "slti",
            Sltiu => "sltiu", Xori => "sltiu", Srli => "srli",
            Srai  => "srai",  Ori  => "ori",   Andi => "andi",

            Add => "add", Sub  => "sub",  Sll => "sll",
            Slt => "slt", Sltu => "sltu", Xor => "xor",
            Srl => "srl", Sra  => "sra",
            Or  => "or",  And  => "and",

            Lb  => "lb",  Lh  => "lh", Lw => "lw",
            Lbu => "lbu", Lhu => "lhu",

            Sb => "sb",
            Sh => "sh", 
            Sw => "sw",

            Beq => "beq", Bne  => "bne",  Blt  => "blt",
            Bge => "bge", Bltu => "bltu", Bgeu => "bgeu",

            Jal  => "jal",
            Jalr => "jalr",

            Lui   => "lui", 
            Auipc => "auipc",

            Fence => "fence",

            Ecall  => "ecall", 
            Ebreak => "ebreak",
        };

        f.pad(op_str)
    }
}
