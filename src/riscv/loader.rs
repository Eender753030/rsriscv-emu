use crate::utils::exception::CliError;

use std::{io::{self, Read}, fs, env};

pub fn load_arg() -> Result<std::iter::Skip<env::Args>, CliError>{
    let args = env::args().skip(1);

    if args.len() == 0 {
        Err(CliError::NoInputBinary)
    } else if args.len() > 1 {
        Err(CliError::TooManyArgument)
    } else {
        Ok(args)
    }
}

pub fn read_binary(filename: &str) -> io::Result<Vec<u8>>{
    let file = fs::File::open(filename)?;
    let mut reader = io::BufReader::new(file);

    let mut content = Vec::new();

    reader.read_to_end(&mut content)?;

    Ok(content)
}