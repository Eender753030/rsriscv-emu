use AOp::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AmoInsData {
    pub rd: u8,
    pub rs1: u8,
    pub rs2: u8,
    pub rl: u8,
    pub aq: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AOp {
    LrW, ScW, 

    AmoSwapW, AmoAddW, AmoAndW, AmoOrW, AmoXorW,
    AmoMaxW, AmoMinW, AmoMaxuW, AmoMinuW,
}

impl AOp {
    pub(crate) fn decode(funct5: u8, funct3: u8, rs2: u8) -> Option<AOp> {
        Some(match funct3 {
            0x2 => match funct5 {
                0x02 if rs2 == 0 => LrW,
                0x03 => ScW,

                0x01 => AmoSwapW,
                0x00 => AmoAddW,
                0x04 => AmoXorW,
                0x0c => AmoAndW,
                0x08 => AmoOrW,
                0x10 => AmoMinW,
                0x14 => AmoMaxW,
                0x18 => AmoMinuW,
                0x1c => AmoMaxuW,
                _    => return None,
            },
            _ => return None,
        })
    }

    pub fn is_load(&self) -> bool {
        self == &LrW
    }
}

impl std::fmt::Display for AOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad( 
            match self {
                LrW      => "lr.w", 
                ScW      => "sc.w",
                AmoSwapW => "amoswap.w",
                AmoAddW  => "amoadd.w",
                AmoXorW  => "amoxor.w",
                AmoAndW  => "amoand.w", 
                AmoOrW   => "amoor.w",
                AmoMinW  => "amomin.w",
                AmoMaxW  => "amomax.w",
                AmoMinuW => "amominu.w",
                AmoMaxuW => "amomaxu.w"
            }
        )
    }
}