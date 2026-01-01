use risc_v_emulator::riscv::RiscV;
use risc_v_emulator::riscv::{loader, parser};
use risc_v_emulator::ui::{self, state};

use anyhow::Result;

fn main() -> Result<()> {
    let mut ins_list = vec![];
    let mut machine = RiscV::default();

    for arg in loader::load_arg()? {
        let code = loader::read_binary(&arg)?;
        machine.load(&code)?;
        ins_list.extend(parser::parse_binary(code));
    }

    let (reg_data, mem_data, pc_num) = machine.dump();

    let mut emu_state = state::EmuState::new(ins_list, reg_data, mem_data, pc_num);

    ui::tui_loop(&mut emu_state, &mut machine)?;

    Ok(())
}