use thiserror::Error;

const USAGE: &str = "Usage: cargo run <.elf>";

#[derive(Error, Debug)]
pub enum CliError {
    #[error("No input file\n{}", USAGE)]
    NoInputFile,

    #[error("Too many input file\n{}", USAGE)]
    TooManyArgument,
}
