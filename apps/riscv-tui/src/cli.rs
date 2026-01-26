use std::env;

use crate::error::CliError;

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

    Ok(match args.len() {
        0   => return Err(CliError::NoInputFile),
        2.. => return Err(CliError::TooManyArgument),
        // Safe: Here we sure args has the first element 
        1   => args.next().unwrap(),
    })
}
