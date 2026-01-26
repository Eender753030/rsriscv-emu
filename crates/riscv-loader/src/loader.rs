//! Handle ELF file load by file path 

mod binary;
mod elf;

use std::path::Path;

use crate::error::LoadError;
use crate::load_info::LoadInfo;

use elf::load_elf;

/// Dispatch `filepath` to `load_elf`
/// Return `LoadInfo` for Risc-V to load into memory
/// If the target file is not ELF file, call `read_binary` instead
/// Other errors will return directly 
/// ## Example
/// ```rust,no_run
/// # use riscv_loader::load;
/// let filepath: String = "file.elf".to_string();
/// 
/// let load_info = load(&filepath).expect("Get LoadInfo successed");
/// ```
pub fn load<P: AsRef<Path>>(filepath: &P) -> Result<LoadInfo, LoadError> {     
    load_elf(filepath).or_else(|e| match e {
        LoadError::NotElfFile(raw_binary) => Ok(LoadInfo::from_raw_binary(raw_binary)),
        _ => Err(e),
    })
}
