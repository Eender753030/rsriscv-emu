#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZifenceiOp {
    FenceI
}

impl ZifenceiOp {
    pub(crate) fn decode(funct3: u8) -> Option<ZifenceiOp> {
        match funct3 {
            0x1 => Some(ZifenceiOp::FenceI),
            _ => None,
        }
    }
}
