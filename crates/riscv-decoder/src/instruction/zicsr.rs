use ZicsrOp::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZicsrOp {
    Csrrw, Csrrs, Csrrc,
    Csrrwi, Csrrsi, Csrrci,
}

impl ZicsrOp {
    pub(crate) fn decode(funct3: u8) -> Option<ZicsrOp> {
        match funct3 {
            0x1 => Some(Csrrw),
            0x2 => Some(Csrrs),
            0x3 => Some(Csrrc),
            0x5 => Some(Csrrwi),
            0x6 => Some(Csrrsi),
            0x7 => Some(Csrrci),
            _   => None
        }
    }

    pub fn is_imm(&self) -> bool {
        matches!(self, Csrrwi | Csrrsi | Csrrci)
    }

    pub fn is_rw(&self) -> bool {
        matches!(self, Csrrw | Csrrwi)
    }

    pub fn is_rs(&self) -> bool {
        matches!(self, Csrrs| Csrrsi)
    }

    pub fn is_rc(&self) -> bool {
        matches!(self, Csrrc| Csrrci)
    }
}

impl std::fmt::Display for ZicsrOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad( 
            match self {
                Csrrw  => "csrrw", 
                Csrrs  => "csrrs", 
                Csrrc  => "csrrc",
                Csrrwi => "csrrwi", 
                Csrrsi => "csrrsi", 
                Csrrci => "csrrci",
            }
        )
    }
}