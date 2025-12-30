use thiserror::Error;

#[derive(Error, Debug)]
pub enum RiscVError {
    #[error("Register: Not exist register: {0}")]
    InvalidRegister(usize),

    #[error("Memeory: Out of bound")]
    OutOfBoundMemory,

    #[error("PC: PC value `{0}` not misaligned to byte")]
    InstructionAddressMisaligned(u32),

    #[error("OpCode: Not implemented opcode: 0x{0:x}")]
    NotImplementedOpCode(u32),

    #[error("OpCode: Not implemented func 0x{0:x} from: 0x{1:x}")]
    NotImplementedFunc(u32, u8),

    #[error("System Call: Not implemented system call: {0}")]
    NotImplementedSysCall(u32),

    #[error("Exit with code {0}")]
    SystemExit(u32),
}