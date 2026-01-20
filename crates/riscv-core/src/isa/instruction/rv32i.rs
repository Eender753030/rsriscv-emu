use crate::isa::opcode::OpCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rv32iOp {
    // Itype
    Addi,
    Slli,
    Slti,
    Sltiu,
    Xori,
    Srli,
    Srai,
    Ori,
    Andi,
    Lb,
    Lh,
    Lw,
    Lbu,
    Lhu,
    Jalr,
    Fence,
    FenceI,
    Ecall,
    Ebreak,
    // Rtype
    Add,
    Sub,
    Sll,
    Slt,
    Sltu,
    Xor,
    Srl,
    Sra,
    Or,
    And,
    // Btype
    Beq,
    Bne,
    Blt,
    Bge,
    Bltu,
    Bgeu,
    // Stype
    Sb,
    Sh,
    Sw,
    // Jtype
    Jal,
    // Utype
    Auipc,
    Lui,
}

impl Rv32iOp {
    pub fn decode_itype(opcode: OpCode, funct3: u8, funct7: u8, funct12: u16) -> Option<Rv32iOp> {
        match opcode {
            OpCode::Itype => match funct3 {
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
                0x1 => Some(Rv32iOp::FenceI),
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

    pub fn decode_rtype(funct3: u8, funct7: u8) -> Option<Rv32iOp> {
        match funct3 {
            0x0 => match funct7 {
                0x00 => Some(Rv32iOp::Add),
                0x20 => Some(Rv32iOp::Sub),
                _ => None,
            },
            0x1 => Some(Rv32iOp::Sll),
            0x2 => Some(Rv32iOp::Slt),
            0x3 => Some(Rv32iOp::Sltu),
            0x4 => Some(Rv32iOp::Xor),
            0x5 => match funct7 {
                0x00 => Some(Rv32iOp::Srl),
                0x20 => Some(Rv32iOp::Sra),
                _ => None,
            },
            0x6 => Some(Rv32iOp::Or),
            0x7 => Some(Rv32iOp::And),
            _ => None,
        }
    }

    pub fn decode_stype(funct3: u8) -> Option<Rv32iOp> {
        match funct3 {
            0x0 => Some(Rv32iOp::Sb),
            0x1 => Some(Rv32iOp::Sh),
            0x2 => Some(Rv32iOp::Sw),
            _ => None,
        }
    }

    pub fn decode_btype(funct3: u8) -> Option<Rv32iOp> {
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

    pub fn decode_jtype() -> Option<Rv32iOp> {
        Some(Rv32iOp::Jal)
    }

    pub fn decode_utype(opcode: OpCode) -> Option<Rv32iOp> {
        match opcode {
            OpCode::UtypeAuipc => Some(Rv32iOp::Auipc),
            OpCode::UtypeLui => Some(Rv32iOp::Lui),
            _ => None,
        }
    }
}
