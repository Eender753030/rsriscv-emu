use crate::opcode::OpCode;

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
        match opcode {
            OpCode::ItypeAr => match funct3 {
                0x0 => Some(Rv32iOp::Addi),
                0x1 => Some(Rv32iOp::Slli),
                0x2 => Some(Rv32iOp::Slti),
                0x3 => Some(Rv32iOp::Sltiu),
                0x4 => Some(Rv32iOp::Xori),
                0x5 => match funct7 {
                    0x00 => Some(Rv32iOp::Srli),
                    0x20 => Some(Rv32iOp::Srai),
                    _ => None,
                },
                0x6 => Some(Rv32iOp::Ori),
                0x7 => Some(Rv32iOp::Andi),
                _ => None,
            },
            OpCode::ItypeLoad => match funct3 {
                0x0 => Some(Rv32iOp::Lb),
                0x1 => Some(Rv32iOp::Lh),
                0x2 => Some(Rv32iOp::Lw),
                0x4 => Some(Rv32iOp::Lbu),
                0x5 => Some(Rv32iOp::Lhu),
                _ => None,
            },
            OpCode::ItypeJump => match funct3 {
                0x0 => Some(Rv32iOp::Jalr),
                _ => None,
            },
            OpCode::ItypeFence => match funct3 {
                0x0 => Some(Rv32iOp::Fence),
                _ => None,
            },
            OpCode::ItypeSystem => match funct3 {
                0x0 => match funct12 {
                    0x000 => Some(Rv32iOp::Ecall),
                    0x001 => Some(Rv32iOp::Ebreak),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        }
    }

    pub(crate) fn decode_rtype(funct3: u8, funct7: u8) -> Option<Rv32iOp> {
        match funct7 {
            0x00 => match funct3 {
                0x0 => Some(Rv32iOp::Add),
                0x1 => Some(Rv32iOp::Sll),
                0x2 => Some(Rv32iOp::Slt),
                0x3 => Some(Rv32iOp::Sltu),
                0x4 => Some(Rv32iOp::Xor),
                0x5 => Some(Rv32iOp::Srl),
                0x6 => Some(Rv32iOp::Or),
                0x7 => Some(Rv32iOp::And),
                _ => None,
            }
            0x20 => match funct3 {
                0x0 => Some(Rv32iOp::Sub),
                0x5 => Some(Rv32iOp::Sra),
                _ => None
            },
            _ => None,
        }
        
    }

    pub(crate) fn decode_stype(funct3: u8) -> Option<Rv32iOp> {
        match funct3 {
            0x0 => Some(Rv32iOp::Sb),
            0x1 => Some(Rv32iOp::Sh),
            0x2 => Some(Rv32iOp::Sw),
            _ => None,
        }
    }

    pub(crate) fn decode_btype(funct3: u8) -> Option<Rv32iOp> {
        match funct3 {
            0x0 => Some(Rv32iOp::Beq),
            0x1 => Some(Rv32iOp::Bne),
            0x4 => Some(Rv32iOp::Blt),
            0x5 => Some(Rv32iOp::Bge),
            0x6 => Some(Rv32iOp::Bltu),
            0x7 => Some(Rv32iOp::Bgeu),
            _ => None,
        }
    }

    pub(crate) fn decode_jtype() -> Option<Rv32iOp> {
        Some(Rv32iOp::Jal)
    }

    pub(crate) fn decode_utype(opcode: OpCode) -> Option<Rv32iOp> {
        match opcode {
            OpCode::UtypeAuipc => Some(Rv32iOp::Auipc),
            OpCode::UtypeLui => Some(Rv32iOp::Lui),
            _ => None,
        }
    }

    pub fn is_itype_ar(&self) -> bool {
        matches!(self, 
                Rv32iOp::Addi | Rv32iOp::Slli |
                Rv32iOp::Slti | Rv32iOp::Sltiu |
                Rv32iOp::Xori | Rv32iOp::Srli |
                Rv32iOp::Srai |  Rv32iOp::Ori |
                Rv32iOp::Andi
        )
    }

    pub fn is_itype_load(&self) -> bool {
        matches!(self, 
            Rv32iOp::Lb | Rv32iOp::Lh | Rv32iOp::Lw
            | Rv32iOp::Lbu | Rv32iOp::Lhu
        )
    }

    pub fn is_itype_jump(&self) -> bool {
        self == &Rv32iOp::Jalr
    }

    pub fn is_itype_system(&self) -> bool {
        matches!(self,
            Rv32iOp::Ecall | Rv32iOp::Ebreak
        )

    }
    pub fn is_itype_fence(&self) -> bool {
        self == &Rv32iOp::Fence
    }

    pub fn is_rtype(&self) -> bool {
        matches!(self, 
            Rv32iOp:: Add | Rv32iOp:: Sub |
            Rv32iOp:: Sll | Rv32iOp:: Slt |
            Rv32iOp:: Sltu | Rv32iOp:: Xor |
            Rv32iOp:: Srl | Rv32iOp:: Sra |
            Rv32iOp:: Or | Rv32iOp:: And
        )
    }

    pub fn is_stype(&self) -> bool {
        matches!(self, 
            Rv32iOp:: Sb| Rv32iOp::Sh | Rv32iOp::Sw
        )
    }

    pub fn is_btype(&self) -> bool {
        matches!(self, 
            Rv32iOp:: Beq | Rv32iOp:: Bne | 
            Rv32iOp:: Blt | Rv32iOp:: Bge |
            Rv32iOp:: Bltu | Rv32iOp:: Bgeu
        )
    }

    pub fn is_jtype(&self) -> bool {
        self == &Rv32iOp::Jal
    }

    pub fn is_utype(&self) -> bool {
        matches!(self, 
            Rv32iOp::Lui | Rv32iOp::Auipc)
    }
}

impl std::fmt::Display for Rv32iOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let op_str = match self {
            Rv32iOp::Addi => "addi", Rv32iOp::Slli => "slli", Rv32iOp::Slti => "slti",
            Rv32iOp::Sltiu => "sltiu", Rv32iOp::Xori => "sltiu", Rv32iOp::Srli => "srli",
            Rv32iOp::Srai => "srai", Rv32iOp::Ori => "ori", Rv32iOp::Andi => "andi",

            Rv32iOp::Add => "add", Rv32iOp::Sub => "sub", Rv32iOp::Sll => "sll",
            Rv32iOp::Slt => "slt", Rv32iOp::Sltu => "sltu", Rv32iOp::Xor => "xor",
            Rv32iOp::Srl => "srl", Rv32iOp::Sra => "sra",
            Rv32iOp::Or => "or", Rv32iOp::And => "and",

            Rv32iOp::Lb => "lb", Rv32iOp::Lh => "lh", Rv32iOp::Lw => "lw",
            Rv32iOp::Lbu => "lbu", Rv32iOp::Lhu => "lhu",

            Rv32iOp::Sb => "sb", Rv32iOp::Sh => "sh", Rv32iOp::Sw => "sw",

            Rv32iOp::Beq => "beq", Rv32iOp::Bne => "bne", Rv32iOp::Blt => "blt",
            Rv32iOp::Bge => "bge", Rv32iOp::Bltu => "bltu", Rv32iOp::Bgeu => "bgeu",

            Rv32iOp::Jal => "jal", Rv32iOp::Jalr => "jalr",

            Rv32iOp::Lui => "lui", Rv32iOp::Auipc => "auipc",

            Rv32iOp::Fence => "fence",

            Rv32iOp::Ecall => "ecall", Rv32iOp::Ebreak => "ebreak",
        };

        f.pad(op_str)
    }
}
