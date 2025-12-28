#[derive(Debug)]

pub enum TypeKind {
    IType,
}

#[derive(Debug)]
pub enum Types {
    IType {imm: i32, rs1: usize, rd: usize, func: u16},
}

impl Types {
    pub fn parse(op_type: TypeKind, ins: u32, func: u16) -> Self {
        match op_type{
            TypeKind::IType => {
                Types::IType {imm: ((ins & 0xfff00000) as i32) >> 20,
                       rs1: ((ins >> 15) & 0x1f) as usize,
                       rd:  ((ins >> 7) & 0x1f) as usize,
                       func}
            }
        }
    }
}