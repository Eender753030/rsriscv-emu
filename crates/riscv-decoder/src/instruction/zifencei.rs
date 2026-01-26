use ZifenceiOp::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZifenceiOp {
    FenceI
}

impl ZifenceiOp {
    pub(crate) fn decode(funct3: u8) -> Option<ZifenceiOp> {
        Some(match funct3 {
            0x1 => FenceI,
            _ => return  None,
        })
    }
}

impl std::fmt::Display for ZifenceiOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad("fence.i")
    }
}
