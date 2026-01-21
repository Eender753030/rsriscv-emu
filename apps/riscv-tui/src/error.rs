use thiserror::Error;

const USAGE: &str = "Usage: cargo run <binary_file>";

#[derive(Error, Debug)]
pub enum CliError {
    #[error("CLI: No input binary file\n{}", USAGE)]
    NoInputBinary,

    #[error("CLI: Too many input file\n{}", USAGE)]
    TooManyArgument,
}