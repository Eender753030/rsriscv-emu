use risc_v_emulator::riscv::RiscV;
use risc_v_emulator::riscv::{loader};
use risc_v_emulator::ui::{self, state};

use anyhow::Result;

// Main entry for Risc-V emulator. Return any errors.
fn main() -> Result<()> {
    let mut machine = RiscV::default();

    // Access binary data and load instructions into Risc-V's instruction memory
    machine.load(
        &loader::read_binary(
            &loader::load_arg()?
        )?
    )?;

    // Dump initial state of Risc-V
    let (reg_data, mem_data, pc_num) = machine.dump_data();

    // Parse the binary and turn into Vec<String> for display instructions
    let ins_list = machine.dump_ins()?;

    // Create mut instant for Risc-V's state. Mut is for changing data and state of Risc-V
    let mut emu_state = state::EmuState::new(ins_list, reg_data, mem_data, pc_num);

    // Go into the TUI display loop
    ui::tui_loop(&mut emu_state, &mut machine)?;

    Ok(())
}