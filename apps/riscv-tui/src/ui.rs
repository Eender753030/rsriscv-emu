pub mod terminal;
pub mod state;
pub mod key;

use ratatui::{
    layout::{Layout, Rect, Constraint, Alignment},
    widgets::{Block, List, ListItem, Paragraph},
    style::{Color, Style},
    Frame
};

use anyhow::Result;

use std::time::Duration;

use crate::ui::state::{Mid, Selected};
use riscv_core::{RiscV, constance::DRAM_BASE_ADDR, debug::*, error::RiscVError};
use key::KeyControl;
use state::{EmuState, EmuMode};


const HEADER: &str = "RsRisc-V Emulator v0.0.1";
const OBSERVATION_HINT_MESSAGE: &str = "Q: Leave    TAB: Switch mode    Up/Down: Scroll    Left/Right: Change panel";
const EMULATE_HINT_MESSAGE: &str = "Q: Leave   TAB: Change mode    S: Single step    P: Run to end    R: Reset";

// const BERKELEY_BLUE: (u8, u8, u8) = (0, 50, 98);
// const CALIFORNIA_GOLD: (u8, u8, u8) = (253, 181, 21);

pub fn tui_loop(machine: &mut RiscV, code: &Vec<u8>) -> Result<()> {
    let mut emu_terminal = terminal::EmuTerminal::new()?;
    let mut emu_state = EmuState::new(machine, code.len() / 4);

    loop {
        emu_terminal.draw(ui, &mut emu_state)?;

        match emu_state.mode {
            EmuMode::Observation => {
                match key::poll_key_event(Duration::from_mins(100))? {
                    KeyControl::Quit => break Ok(()),
                    KeyControl::ChangeMode => {
                        emu_state.running_mode_selected();
                        emu_state.mode = EmuMode::Stay;
                    }
                    KeyControl::GoNext => emu_state.next(),
                    KeyControl::GoPrev => emu_state.prev(),
                    KeyControl::GoLeft => emu_state.go_left(),
                    KeyControl::GoRight => emu_state.go_right(),
                    KeyControl::ChangeMid => emu_state.change_mid(),
                    _ => {},
                }
            },
            EmuMode::Stay =>  {
                match key::poll_key_event(Duration::from_mins(100))? {
                    KeyControl::Quit => break Ok(()),
                    KeyControl::ChangeMode => {
                        emu_state.observation_mode_selected();
                        emu_state.mode = EmuMode::Observation;
                    }
                    KeyControl::Reset => {
                        emu_state.machine.reset();
                        emu_state.machine.fisrt_load(code)?;
                        emu_state.update_data();
                    },
                    KeyControl::Step => {
                        if let Err (e) = emu_state.machine.step() {
                            match e {
                                RiscVError::EndOfInstruction => {},
                                _ => return Err(anyhow::Error::new(e)),
                            }
                        }
                        emu_state.update_data();
                    },
                    KeyControl::RunToEnd => emu_state.mode = EmuMode::Running,
                    _ => {},
                }
            },
            EmuMode::Running => {  
                if (emu_state.pc - DRAM_BASE_ADDR) as usize >= code.len() {
                    emu_state.mode = EmuMode::Stay;
                } else {
                    if let Err (e) = emu_state.machine.step() {
                        match e {
                            RiscVError::EndOfInstruction => emu_state.mode = EmuMode::Stay,
                            _ => return Err(anyhow::Error::new(e)),
                        }
                    }
                    emu_state.update_data();
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

fn render_content<D: DebugInterface>(f: &mut Frame, area: Rect, emu_state: &mut EmuState<D>) {
    let info_layout = Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Percentage(20),
        Constraint::Percentage(30),
    ]).split(area);

    render_ins(f, info_layout[0], emu_state);
    render_mid(f, info_layout[1], emu_state);
    render_mem(f, info_layout[2], emu_state);
}

fn render_ins<D: DebugInterface>(f: &mut Frame, area: Rect, emu_state: &mut EmuState<D>) {
    let items: Vec<ListItem> = emu_state.ins.list.iter().map(|(addr, ins)| {
        let marker = if emu_state.pc == *addr {
            "PC >>"
        } else {
            "     "
        };
        ListItem::new(format!("{} {:#08x}: {}", marker, addr, ins))
    }).collect();
    let highlight_color = match &emu_state.selected {
        Selected::Ins => (Color::Rgb(242, 242, 242), Color::Rgb(0, 50, 98)),
        _ => (Color::Rgb(0, 50, 98), Color::Rgb(253, 181, 21))
    };

    let list = List::new(items)
        .block(Block::bordered().title("Instruction"))
        .style(Style::default().bg(Color::Rgb(0, 50, 98)).fg(Color::Rgb(253, 181, 21)))
        .highlight_style(Style::default().bg(highlight_color.0).fg(highlight_color.1));

    f.render_stateful_widget(list, area, &mut emu_state.ins.list_state);
}

fn render_mid<D: DebugInterface>(f: &mut Frame, area: Rect, emu_state: &mut EmuState<D>) {
    let items: Vec<ListItem> =  match emu_state.mid_selected {
        Mid::Reg => {
            emu_state.reg.list.iter().enumerate().map(|(i, data)| {
                ListItem::new(format!(" x{:<2}: {}", i, data))
            }).collect()
        },
        Mid::Csr => {
            emu_state.csr.list.iter().map(|(name, data)| {
                ListItem::new(format!(" {:<7}: {}", name, data))
            }).collect()
        },
    };

    let highlight_color = match (&emu_state.selected, &emu_state.mode) {
        (Selected::Mid(_), EmuMode::Observation) => (Color::Rgb(242, 242, 242), Color::Rgb(0, 50, 98)),
        _ => (Color::Rgb(0, 50, 98), Color::Rgb(253, 181, 21))
    };

    let list = List::new(items)
        .block(Block::bordered().title("Register / Csr (Press C to change)"))
        .style(Style::default().bg(Color::Rgb(0, 50, 98)).fg(Color::Rgb(253, 181, 21)))
        .highlight_style(Style::default().bg(highlight_color.0).fg(highlight_color.1));

    f.render_stateful_widget(list, area, &mut emu_state.reg.list_state);
}

fn render_mem<D: DebugInterface>(f: &mut Frame, area: Rect, emu_state: &mut EmuState<D>) {
    let items: Vec<ListItem> = emu_state.mem.list.chunks(4).enumerate()
        .map(|(i, data)| {
            ListItem::new(format!(" {:#08x}: {}", i * 16, 
                data.iter().map(|d| format!("{:02x} ", d)).collect::<String>()))
        }).collect();

    let highlight_color = match (&emu_state.selected, &emu_state.mode) {
        (Selected::Mem, EmuMode::Observation) => (Color::Rgb(242, 242, 242), Color::Rgb(0, 50, 98)),
        _ => (Color::Rgb(0, 50, 98), Color::Rgb(253, 181, 21))
    };

    let list = List::new(items)
        .block(Block::bordered().title("Dram"))
        .style(Style::default().bg(Color::Rgb(0, 50, 98)).fg(Color::Rgb(253, 181, 21)))
        .highlight_style(Style::default().bg(highlight_color.0).fg(highlight_color.1));

    f.render_stateful_widget(list, area, &mut emu_state.mem.list_state);
}