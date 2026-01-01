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

use crate::utils::exception::RiscVError;
use crate::riscv::RiscV;
use key::KeyControl;
use state::{EmuState, EmuMode};

const HEADER: &str = "RsRisc-V Emulator v0.0.1";
const OBSERVATION_HINT_MESSAGE: &str = "Q: Leave    TAB: Switch mode    Up/Down: Scroll    Left/Right: Change panel";
const EMULATE_HINT_MESSAGE: &str = "Q: Leave   TAB: Change mode    S: Single step    P: Run to end    R: Reset";

pub fn tui_loop(emu_state: &mut EmuState, machine: &mut RiscV) -> Result<()> {
    let mut emu_terminal = terminal::EmuTerminal::init_terminal()?;

    loop {
        emu_terminal.draw(ui, emu_state)?;

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
                        machine.reset();
                        emu_state.update_data(machine.dump());
                        emu_state.update_ins_selected();
                    },
                    KeyControl::Step => {
                        if let Err(e) = machine.step() {
                            match e {
                                RiscVError::SystemExit(_) | RiscVError::EndOfInstruction => {},
                                _ => return Err(anyhow::Error::new(e))
                            }
                        }
                        emu_state.update_data(machine.dump());
                        emu_state.update_ins_selected();
                    },
                    KeyControl::RunToEnd => emu_state.mode = EmuMode::Running,
                    _ => {},
                }
            },
            EmuMode::Running => {
                if let Err(e) = machine.step() {
                    match e {
                        RiscVError::SystemExit(_) | RiscVError::EndOfInstruction => emu_state.mode = EmuMode::Stay,
                        _ => return Err(anyhow::Error::new(e))
                    }
                }
                emu_state.update_data(machine.dump());
                emu_state.update_ins_selected();
            }
        }
    }
}

pub fn ui(f: &mut Frame, emu_state: &mut EmuState) {
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

fn render_header(f: &mut Frame, area: Rect, emu_state: &EmuState) {
    let message = match emu_state.mode {
        EmuMode::Observation => "Observation Mode",
        EmuMode::Stay | EmuMode::Running => "Emulate Mode"
    };
    
    let paragraph = Paragraph::new(message)
        .block(Block::bordered().title(HEADER))
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

fn render_paragraph(f: &mut Frame, area: Rect, message: &str) {
    let paragraph = Paragraph::new(message)
        .block(Block::bordered())
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

fn render_content(f: &mut Frame, area: Rect, emu_state: &mut EmuState) {
    let info_layout = Layout::horizontal([
        Constraint::Percentage(40),
        Constraint::Percentage(30),
        Constraint::Percentage(30),
    ]).split(area);

    render_ins(f, info_layout[0], emu_state);
    render_reg(f, info_layout[1], emu_state);
    render_mem(f, info_layout[2], emu_state);
}

fn render_ins(f: &mut Frame, area: Rect, emu_state: &mut EmuState) {
    let items: Vec<ListItem> = emu_state.ins.list.iter().enumerate().map(|(i, ins)| {
        ListItem::new(format!("{:>4}. {}", i + 1, ins))
    }).collect();

    let list = List::new(items)
        .block(Block::bordered().title("Instruction"))
        .highlight_style(Style::default().fg(Color::Black).bg(Color::White));

    f.render_stateful_widget(list, area, &mut emu_state.ins.list_state);
}

fn render_reg(f: &mut Frame, area: Rect, emu_state: &mut EmuState) {
    let items: Vec<ListItem> = emu_state.reg.list.iter().enumerate().map(|(i, data)| {
        ListItem::new(format!(" x{:<2}: {}", i, data))
    }).collect();

    let list = List::new(items)
        .block(Block::bordered().title("Register"))
        .highlight_style(Style::default().fg(Color::Black).bg(Color::White));

    f.render_stateful_widget(list, area, &mut emu_state.reg.list_state);
}

fn render_mem(f: &mut Frame, area: Rect, emu_state: &mut EmuState) {
    let items: Vec<ListItem> = emu_state.mem.list.iter().enumerate().map(|(i, data)| {
        ListItem::new(format!(" 0x{:06x}: {:02x?}", i * 4, data))
    }).collect();

    let list = List::new(items)
        .block(Block::bordered().title("Memory"))
        .highlight_style(Style::default().fg(Color::Black).bg(Color::White));

    f.render_stateful_widget(list, area, &mut emu_state.mem.list_state);
}