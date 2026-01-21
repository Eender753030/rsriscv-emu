mod loader;
mod ui;

pub mod error;

use riscv_core::RiscV;

use anyhow::Result;

// Main entry for Risc-V emulator. Return any errors.
fn main() -> Result<()> {
    let mut machine = RiscV::default();

    let code = loader::read_binary(&loader::load_arg()?)?;

    // Access binary data and load instructions into Risc-V's instruction memory
    machine.fisrt_load(&code)?;

    // Go into the TUI display loop
    ui::tui_loop(&mut machine, &code)?;

    if cfg!(debug_assertions) {
        println!("{:?}", machine);
    }   

    Ok(())
}
