use crate::Exception;

use PrivilegeMode::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PrivilegeMode {
    User       = 0,
    #[cfg(feature = "s")]
    Supervisor = 1,
    #[default]
    Machine    = 3,
}

impl From<u8> for PrivilegeMode {
    fn from(value: u8) -> Self {
        match value {
            0b00 => User,
            #[cfg(feature = "s")] 0b01 => Supervisor,
            _    => Machine,
        }
    }
}

impl PrivilegeMode {
    pub fn call_exception(&self) -> Exception {
        match self {
            User       => Exception::EnvironmentCallFromUMode,
            #[cfg(feature = "s")] Supervisor => Exception::EnvironmentCallFromSMode,
            Machine    => Exception::EnvironmentCallFromMMode,
        }
    }
}
