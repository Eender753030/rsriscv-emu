#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MOp {
    Mul, Mulh, Mulhu, Mulhsu,
    Div, Divu, Rem, Remu,
}

impl MOp {
    pub(crate) fn decode(funct3: u8, funct7: u8) -> Option<MOp> {
        match funct7 {
            0x01 => match funct3 {
                0x0 => Some(MOp::Mul),
                0x1 => Some(MOp::Mulh),
                0x2 => Some(MOp::Mulhsu),
                0x3 => Some(MOp::Mulhu),
                0x4 => Some(MOp::Div),
                0x5 => Some(MOp::Divu),
                0x6 => Some(MOp::Rem),
                0x7 => Some(MOp::Remu),
                _ => None
            }
            _ => None,
        }
    }
}

impl std::fmt::Display for MOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad( 
            match self {
                MOp::Mul => "mul", MOp::Mulh => "mulh",
                MOp::Mulhsu => "mulhsu", MOp::Mulhu => "mulhu",
                MOp::Div => "div", MOp::Divu => "divu",
                MOp::Rem => "rem", MOp::Remu => "remu",
            }
        )
    }
}