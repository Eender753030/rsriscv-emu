use risc_v_emulator::RiscVError;
use risc_v_emulator::riscv::RiscV;
use risc_v_emulator::riscv::loader;

use std::env;

const USAGE: &str = "Usage: cargo run <binary_file> [binary_file] ...";

fn main() {
    let args = env::args().skip(1);

    if args.len() == 0 {
        eprintln!("Error: No input file\n{}", USAGE);
        std::process::exit(1);
    }

    let mut machine = RiscV::default();

    for arg in args {
        match loader::read_binary(&arg) {
            Ok(code) => {
                if let Err(e) = machine.cycle(&code) {
                    match e {
                        RiscVError::SystemExit(_) => println!("{}", e),
                        _ => eprintln!("Error: {}", e)
                    }
                }
            },
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    machine.print();
}