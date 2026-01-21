#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZicsrOp {
    Csrrw,
    Csrrs,
    Csrrc,
    Csrrwi,
    Csrrsi,
    Csrrci,
    Mret,
}

impl ZicsrOp {
    pub fn decode(funct3: u8) -> Option<ZicsrOp> {
        match funct3 {
            0x1 => Some(ZicsrOp::Csrrw),
            0x2 => Some(ZicsrOp::Csrrs),
            0x3 => Some(ZicsrOp::Csrrc),
            0x5 => Some(ZicsrOp::Csrrwi),
            0x6 => Some(ZicsrOp::Csrrsi),
            0x7 => Some(ZicsrOp::Csrrci),
            _ => None
        }
    }

    pub fn decode_ret(raw: u32) -> Option<ZicsrOp> {
        match raw {
            0x30200073 => Some(ZicsrOp::Mret),
            _ => None
        }
    }
}
