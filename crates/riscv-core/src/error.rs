use thiserror::Error;

#[derive(Error, Clone, Copy, Debug, PartialEq)]
pub enum RiscVError {
    #[error("Register: Not exist register: {0}")]
    InvalidRegister(u8),

    #[error("Memeory: Out of bound")]
    OutOfBoundMemory,

    #[error("Memeory: Only can read 1 to 4 bytes")]
    ReadInvalidBytes,

    #[error("Memeory: Cannot write 0 byte")]
    WriteInvalidBytes,

    #[error("PC: PC value `{0}` not misaligned to byte")]
    AddressMisaligned(u32),

    #[error("OpCode: Not implemented funct 0x{0:x} from: 0x{1:x}")]
    NotImplementedFunc(u8, u8),

    #[error("System Call: Not implemented system call: {0}")]
    NotImplementedSysCall(u32),

    #[error("Exit with code {0}")]
    SystemExit(u32),

    #[error("Reach end of Instructions")]
    EndOfInstruction,

    #[error("The address is not connent yet")]
    UnvalidBusMapping,

    #[error("The address {0} of ram is uninit")]
    ReadUninitAddr(usize),

    #[error("Not Implemet CSR from address {0}")]
    NotImplementedCsr(u16),
}
