mod app;
mod cli;
mod event;
mod state;
mod ui;

use anyhow::Result;

use riscv_loader::load;

use crate::app::EmuApp;

// Main entry for Risc-V emulator. Return any errors.
fn main() -> Result<()> {
    let filepath = &cli::load_arg()?;

    // Access file and load instructions into Risc-V's instruction memory
    let info = load(filepath)?;

    let mut app = EmuApp::new(info)?;
    
    // Go into the TUI app loop
    app.run()?;

    Ok(())
}
