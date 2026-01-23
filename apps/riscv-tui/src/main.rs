mod cli;
mod ui;

pub mod error;

use riscv_core::RiscV;
use riscv_disasm::disasm;
use riscv_loader::load;

use anyhow::Result;

// Main entry for Risc-V emulator. Return any errors.
fn main() -> Result<()> {
    let filepath = &cli::load_arg()?;

    let mut machine = RiscV::default();

    // Access file and load instructions into Risc-V's instruction memory
    let info = load(filepath)?;

    for (code, addr) in info.code.iter() {
        machine.load(*addr, code)?
    }
    machine.set_pc(info.pc_entry);
    if let Some(data_vec) = &info.data {
        for (data, addr) in data_vec.iter() {
            machine.load(*addr, data)?
        }
    }
    if let Some((start, size)) = &info.bss {
        machine.set_mem_zero(*start, *size)?
    }
    if let Some(other_vec) = &info.other {
        for (data, addr) in other_vec.iter() {
            machine.load(*addr, data)?
        }
    }
    let ins_list = disasm::disassembler(&info);
    
    let (code, u32) = &info.code[0];
    // Go into the TUI display loop
    ui::tui_loop(&mut machine, code, *u32, ins_list)?;

    if cfg!(debug_assertions) {
        println!("{:?}", machine);
        println!("{:?}", info);
    }   

    Ok(())
}
