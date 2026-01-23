#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PrivilegeMode {
    #[default]
    Machine = 3,
    Supervisor = 1,
    User = 0,
}

impl From<u8> for PrivilegeMode {
    fn from(value: u8) -> Self {
        match value {
            0b00 => PrivilegeMode::User,
            0b01 => PrivilegeMode::Supervisor,
            _ => PrivilegeMode::Machine,
        }
    }
}