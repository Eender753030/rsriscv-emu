use crate::Exception;

use PrivilegeMode::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PrivilegeMode {
    User       = 0,
    Supervisor = 1,
    #[default]
    Machine    = 3,
}

impl From<u8> for PrivilegeMode {
    fn from(value: u8) -> Self {
        match value {
            0b00 => User,
            0b01 => Supervisor,
            _    => Machine,
        }
    }
}

impl PrivilegeMode {
    pub fn call_exception(&self) -> Exception {
        match self {
            User       => Exception::EnvironmentCallFromUMode,
            Supervisor => Exception::EnvironmentCallFromSMode,
            Machine    => Exception::EnvironmentCallFromMMode,
        }
    }
}
