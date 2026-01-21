//! For handle file or argument from outside.

use crate::error::CliError;

use std::env;
use std::io::{self, Read};
use std::fs;

/// Load CLI argument from `env::args().skip(1)`. Only accept one binary file for now.
/// ## Example
/// ```bash
/// # Here is bash
/// cargo run binary_file
/// ```
/// ```rust,no_run
/// // Rust
/// # use risc_v_emulator::riscv::loader;
/// if let Ok(binary_file_name) = loader::load_arg() {
///     assert_eq!(binary_file_name, String::from("binary_file"));
/// }
/// ```
pub fn load_arg() -> Result<String, CliError>{
    let mut args = env::args().skip(1);

    if args.len() == 0 {
        Err(CliError::NoInputBinary)
    } else if args.len() > 1 {
        Err(CliError::TooManyArgument)
    } else {
        // Safe: Here we sure args has the first element
        Ok(args.next().unwrap())
    }
}

/// Access binary file of `filename` and return its content by `Vec<u8>`.
/// Risc-V is Little Endian.
/// ## Example
/// ```bin
/// ```
/// ```rust,no_run
/// # use risc_v_emulator::riscv::loader;
/// let filename: &str = "binary_file";
/// 
/// let binary_content = loader::read_binary(filename);
/// ```
pub fn read_binary(filename: &str) -> io::Result<Vec<u8>>{
    let file = fs::File::open(filename)?;
    let mut reader = io::BufReader::new(file);

    let mut content = Vec::new();

    reader.read_to_end(&mut content)?;

    Ok(content)
}