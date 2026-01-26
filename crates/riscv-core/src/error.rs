use thiserror::Error;

#[derive(Error, Clone, Copy, Debug, PartialEq, Eq)]
pub enum RiscVError {
    #[error("Can not load data")]
    LoadFailed,

    #[error("Can not set zero in memory")]
    BssInitFailed,
}
