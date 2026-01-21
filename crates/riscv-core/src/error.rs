use thiserror::Error;

#[derive(Error, Clone, Copy, Debug, PartialEq)]
pub enum RiscVError {
    #[error("Exit with code {0}")]
    SystemExit(u32),

    #[error("Reach end of Instructions")]
    EndOfInstruction,
}
