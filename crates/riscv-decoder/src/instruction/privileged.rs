use PrivilegeOp::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivilegeOp {
    Mret, Sret,
    SfenceVma(u32),
}

impl PrivilegeOp {
    pub(crate) fn decode(raw:u32, funct3: u8, funct7: u8, rd: u8) -> Option<PrivilegeOp> {
        Some(match raw {
            0x10200073 => Sret,
            0x30200073 => Mret,
            _          => match funct3 {
                0x0 => match funct7 {
                    0x09 if rd == 0 => {
                        SfenceVma(raw)
                    },
                    _ => return None,
                },
                _ => return None,
            }
        })
    }

    pub fn is_fence(&self) -> bool {
        matches!(self, SfenceVma(_))
    }
}

impl std::fmt::Display for PrivilegeOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad( 
            match self {
                Mret         => "mret",
                Sret         => "sret",
                SfenceVma(_) => "sfence.vma",
            }
        )
    }
}