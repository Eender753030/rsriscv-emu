//! Handle binary related load

use std::fs;
use std::io::{self, Read};
use std::path::Path;

use crate::error::LoadError;

/// Access binary file by `filepath` and return its content by `Vec<u8>` if successed
/// Otherwise, return `LoadError`
/// Risc-V is Little Endian.
pub fn read_binary<P: AsRef<Path>>(filepath: &P) -> Result<Vec<u8>, LoadError>{
    // Try to open file
    let file = fs::File::open(filepath).map_err(|e| 
        LoadError::OpenFileFailed(e.to_string())
    )?;
    
    let mut reader = io::BufReader::new(file);

    let mut content = Vec::new();

    // Read all into `Vec`
    // If io error raise then map to `LoadError` and keep error message
    reader.read_to_end(&mut content).map_err(|e|
        LoadError::ReadRawBinaryFailed(e.to_string())
    )?;

    Ok(content)
}