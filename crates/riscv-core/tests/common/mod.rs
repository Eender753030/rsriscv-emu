use std::path::Path;

use riscv_core::RiscV;
use riscv_core::debug::DebugInterface; 
use riscv_loader;

const MAX_CYCLES: u64 = 1_000_000;

pub fn run_test_file(path: &Path) {
    let filename = path.file_name().unwrap().to_string_lossy();
    println!("Running Test: {}", filename);

    let info = riscv_loader::load(&path).expect("Failed to load ELF file");

    let mut machine = RiscV::default();

    machine.load_info(&info).expect("Failed to load ELF info");

    let tohost_addr = info.symbols
        .as_ref()
        .and_then(|sym| sym.iter().find(|(_, name)| *name == "tohost"))
        .map(|(addr, _)| *addr)
        .expect("ELF missing 'tohost' symbol");

    for cycle in 0..MAX_CYCLES {
        machine.step().expect(&format!("CPU Fault at cycle {}", cycle));

        // Check tohost
        let mem_bytes = machine.inspect_mem(tohost_addr, 4);
        let val = u32::from_le_bytes(mem_bytes.try_into().unwrap());

        if val != 0 {
            if val == 1 {
                // PASS
                println!("\x1b[32mPASS\x1b[0m: {}", filename);
                return;
            } else {
                // FAIL
                panic!("\x1b[31mFAIL\x1b[0m: {} failed with code {} at cycle {}", filename, val, cycle);
            }
        }
    }

    panic!("TIMEOUT: {} exceeded {} cycles", filename, MAX_CYCLES);
}