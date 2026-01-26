mod component;
mod terminal;
mod state;
mod key;

use std::time::Duration;

use anyhow::Result;
use ratatui::{
    Frame,
    layout::{Layout, Rect, Constraint, Alignment},
    widgets::{Block, Paragraph},
    style::{Color, Style},  
};

use riscv_core::RiscV;
use riscv_core::constance::DRAM_BASE_ADDR;
use riscv_core::debug::DebugInterface;

use crate::ui::state::Mid;

use component::*;
use key::KeyControl;
use state::{EmuState, EmuMode};

const HEADER: &str = concat!("RsRisc-V Emulator v", env!("CARGO_PKG_VERSION"));
const OBSERVATION_HINT_MESSAGE: &str = "Q: Leave    TAB: Switch mode    Up/Down: Scroll    Left/Right: Change panel ]/[: Change Dram page";
const EMULATE_HINT_MESSAGE: &str = "Q: Leave   TAB: Change mode    S: Single step    P: Run to end / Stop    R: Reset";

pub fn tui_loop(machine: &mut RiscV, code: &[u8], addr: u32, ins_list: Vec<String>) -> Result<()> {
    let mut emu = EmuState::new(machine, code.len() / 4, ins_list);
    let mut terminal = terminal::EmuTerminal::new()?;
    
    loop {
        terminal.draw(ui, &mut emu)?;

        match emu.mode {
            EmuMode::Observation => {
                match key::poll_key_event(Duration::from_mins(100))? {
                    KeyControl::Quit => break Ok(()),
                    KeyControl::ChangeMode => {
                        emu.running_mode_selected();
                        emu.mode = EmuMode::Stay;
                    }
                    KeyControl::GoNext => emu.next(),
                    KeyControl::GoPrev => emu.prev(),
                    KeyControl::GoLeft => emu.go_left(),
                    KeyControl::GoRight => emu.go_right(),
                    KeyControl::NextPage => emu.next_page(),
                    KeyControl::PrevPage => emu.prev_page(),
                    KeyControl::ChangeMid => emu.change_mid(),
                    _ => {},
                }
            },
            EmuMode::Stay =>  {
                match key::poll_key_event(Duration::from_millis(100))? {
                    KeyControl::Quit => break Ok(()),
                    KeyControl::ChangeMid => emu.change_mid(),
                    KeyControl::ChangeMode => {
                        emu.observation_mode_selected();
                        emu.mode = EmuMode::Observation;
                    }
                    KeyControl::Reset => {
                        emu.machine.reset();
                        emu.machine.load(addr, code)?;
                        emu.update_data();
                        emu.running_mode_selected_update();
                    },
                    KeyControl::Step => {
                        emu.machine.step()?;
                        
                        emu.update_data();
                        emu.running_mode_selected_update();
                    },
                    KeyControl::RunToEnd => emu.mode = EmuMode::Running,
                    _ => {},
                }
            },
            EmuMode::Running => {  
                match key::poll_key_event(Duration::from_millis(16))? {
                    KeyControl::Quit => break Ok(()),
                    KeyControl::RunToEnd => emu.mode = EmuMode::Stay,
                    KeyControl::ChangeMode => {
                        emu.observation_mode_selected();
                        emu.mode = EmuMode::Observation;
                    }
                    _ => {},
                }
                if (emu.pc - DRAM_BASE_ADDR) as usize >= code.len() {
                    emu.mode = EmuMode::Stay;
                } else {
                    emu.machine.step()?;

                    emu.update_data();
                    emu.running_mode_selected_update();
                }
            }
        }
    }
}

pub fn ui<D: DebugInterface>(f: &mut Frame, emu_state: &mut EmuState<D>) {
    let main_layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ]).split(f.area());

    render_header(f, main_layout[0], emu_state);
    render_content(f, main_layout[1], emu_state);
    match emu_state.mode {
        EmuMode::Observation => render_paragraph(f, main_layout[2], OBSERVATION_HINT_MESSAGE),
        EmuMode::Stay | EmuMode::Running => render_paragraph(f, main_layout[2], EMULATE_HINT_MESSAGE)
    }
}

fn render_header<D: DebugInterface>(f: &mut Frame, area: Rect, emu_state: &EmuState<D>) {
    let message = match emu_state.mode {
        EmuMode::Observation => "Observation Mode",
        EmuMode::Stay | EmuMode::Running => "Emulate Mode"
    };
    
    let paragraph = Paragraph::new(message)
        .block(Block::bordered().title(HEADER))
        .style(Style::default().bg(Color::Rgb(0, 50, 98)).fg(Color::Rgb(253, 181, 21)))
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

fn render_paragraph(f: &mut Frame, area: Rect, message: &str) {
    let paragraph = Paragraph::new(message)
        .block(Block::bordered())
        .style(Style::default().bg(Color::Rgb(0, 50, 98)).fg(Color::Rgb(253, 181, 21)))
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

fn render_content<D: DebugInterface>(f: &mut Frame, area: Rect, emu: &mut EmuState<D>) {
    let info_layout = Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Percentage(20),
        Constraint::Percentage(30),
    ]).split(area);

    Instruction::render(f, info_layout[0], emu);
    match emu.mid_selected {
        Mid::Reg => Register::render(f, info_layout[1], emu),
        Mid::Csr => Csr::render(f, info_layout[1], emu),
    }
    Memory::render(f, info_layout[2], emu);
}
