use MOp::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MOp {
    Mul, Mulh, Mulhu, Mulhsu,
    Div, Divu, Rem, Remu,
}

impl MOp {
    pub(crate) fn decode(funct3: u8, funct7: u8) -> Option<MOp> {
        Some(match funct7 {
            0x01 => match funct3 {
                0x0 => Mul,
                0x1 => Mulh,
                0x2 => Mulhsu,
                0x3 => Mulhu,
                0x4 => Div,
                0x5 => Divu,
                0x6 => Rem,
                0x7 => Remu,
                _   => return None
            }
            _ => return None,
        })
    }
}

impl std::fmt::Display for MOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad( 
            match self {
                Mul    => "mul", 
                Mulh   => "mulh",
                Mulhsu => "mulhsu",
                Mulhu  => "mulhu",
                Div    => "div", 
                Divu   => "divu",
                Rem    => "rem",
                Remu   => "remu",
            }
        )
    }
}