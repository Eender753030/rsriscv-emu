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
    BusPopup,
}

impl EmuMode {
    pub fn observation(&mut self) {
        *self = EmuMode::Observation;
    }

    pub fn stay(&mut self) {
        *self = EmuMode::Stay;
    }

    pub fn run(&mut self) {
        *self = EmuMode::Running;
    }

    pub fn popup(&mut self) {
        *self = EmuMode::BusPopup;
    }

    pub fn change_mode(&mut self) {
        *self = match self {
            EmuMode::Observation => EmuMode::Stay,
            EmuMode::Running | EmuMode::Stay | EmuMode::BusPopup => EmuMode::Observation,
        }
    }
}