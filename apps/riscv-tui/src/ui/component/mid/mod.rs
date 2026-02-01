#[cfg(feature = "zicsr")]
pub mod csr;
pub mod register;

#[cfg(not(feature = "zicsr"))]
const MID_TITLE: &str = "Reg (H) Dec/Hex";
#[cfg(feature = "zicsr")]
const MID_TITLE: &str = "(C) Reg / Csr (H) Dec/Hex ";
