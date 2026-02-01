#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mid {
    #[default]
    Reg,
    #[cfg(feature = "zicsr")] Csr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DataView {
    #[default]
    Decimal,
    Hex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Selected {
    #[default]
    Ins,
    Mid(Mid),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EmuMode {
    #[default]
    Observation,
    Stay,
    Running,
}