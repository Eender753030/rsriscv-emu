pub mod terminal;
pub mod state;
pub mod key;

use std::time::Duration;

use anyhow::Result;
use ratatui::{
    Frame,
    layout::{Layout, Rect, Constraint, Alignment},
    widgets::{Block, List, ListItem, Paragraph},
    style::{Color, Style},  
};

use riscv_core::RiscV;
use riscv_core::constance::{DRAM_BASE_ADDR, PAGE_SIZE};
use riscv_core::debug::DebugInterface;

use crate::ui::state::{Mid, Selected};

use key::KeyControl;
use state::{EmuState, EmuMode};


const HEADER: &str = concat!("RsRisc-V Emulator v", env!("CARGO_PKG_VERSION"));
const OBSERVATION_HINT_MESSAGE: &str = "Q: Leave    TAB: Switch mode    Up/Down: Scroll    Left/Right: Change panel ]/[: Change Dram page";
const EMULATE_HINT_MESSAGE: &str = "Q: Leave   TAB: Change mode    S: Single step    P: Run to end / Stop    R: Reset";

// const BERKELEY_BLUE: (u8, u8, u8) = (0, 50, 98);
// const CALIFORNIA_GOLD: (u8, u8, u8) = (253, 181, 21);

pub fn tui_loop(machine: &mut RiscV, code: &[u8], addr: u32, ins_list: Vec<String>) -> Result<()> {
    let mut emu_state = EmuState::new(machine, code.len() / 4, ins_list);
    let mut emu_terminal = terminal::EmuTerminal::new()?;
    
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
                    KeyControl::NextPage => emu_state.next_page(),
                    KeyControl::PrevPage => emu_state.prev_page(),
                    KeyControl::ChangeMid => emu_state.change_mid(),
                    _ => {},
                }
            },
            EmuMode::Stay =>  {
                match key::poll_key_event(Duration::from_millis(100))? {
                    KeyControl::Quit => break Ok(()),
                    KeyControl::ChangeMid => emu_state.change_mid(),
                    KeyControl::ChangeMode => {
                        emu_state.observation_mode_selected();
                        emu_state.mode = EmuMode::Observation;
                    }
                    KeyControl::Reset => {
                        emu_state.machine.reset();
                        emu_state.machine.load(addr, code)?;
                        emu_state.update_data();
                        emu_state.running_mode_selected_update();
                    },
                    KeyControl::Step => {
                        emu_state.machine.step()?;
                        
                        emu_state.update_data();
                        emu_state.running_mode_selected_update();
                    },
                    KeyControl::RunToEnd => emu_state.mode = EmuMode::Running,
                    _ => {},
                }
            },
            EmuMode::Running => {  
                match key::poll_key_event(Duration::from_millis(16))? {
                    KeyControl::Quit => break Ok(()),
                    KeyControl::RunToEnd => emu_state.mode = EmuMode::Stay,
                    KeyControl::ChangeMode => {
                        emu_state.observation_mode_selected();
                        emu_state.mode = EmuMode::Observation;
                    }
                    _ => {},
                }
                if (emu_state.pc - DRAM_BASE_ADDR) as usize >= code.len() {
                    emu_state.mode = EmuMode::Stay;
                } else {
                    emu_state.machine.step()?;

                    emu_state.update_data();
                    emu_state.running_mode_selected_update();
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
    let mut offset = 0;

    let items: Vec<ListItem> = emu_state.ins.list.iter().enumerate()
        .map(|(i, ins)| {
        let marker = if ins.ends_with(':') {
            offset += 1;
            ""
        } else if (emu_state.pc - DRAM_BASE_ADDR) / 4 == ((i - offset) as u32) {
            if emu_state.mode != EmuMode::Observation {
                emu_state.ins.list_state.select(Some(i));
            }
            "PC >>"
        } else {
            "     "
        };
        ListItem::new(format!("{}{}", marker, ins))
    }).collect();
    let highlight_color = match (&emu_state.selected , &emu_state.mode) {
        (Selected::Ins, EmuMode::Observation) => (Color::Rgb(242, 242, 242), Color::Rgb(0, 50, 98)),
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
            ListItem::new(format!(" {:#010x}: {}", i * 16 + (emu_state.page_selected * PAGE_SIZE), 
                data.iter().map(|d| format!("{:02x} ", d)).collect::<String>()))
        }).collect();

    let highlight_color = match (&emu_state.selected, &emu_state.mode) {
        (Selected::Mem, EmuMode::Observation) => (Color::Rgb(242, 242, 242), Color::Rgb(0, 50, 98)),
        _ => (Color::Rgb(0, 50, 98), Color::Rgb(253, 181, 21))
    };

    let list = List::new(items)
        .block(Block::bordered().title(format!("Dram [{:#010x} - {:#010x}]", emu_state.page_selected * PAGE_SIZE, (emu_state.page_selected + 1) * PAGE_SIZE - 1)))
        .style(Style::default().bg(Color::Rgb(0, 50, 98)).fg(Color::Rgb(253, 181, 21)))
        .highlight_style(Style::default().bg(highlight_color.0).fg(highlight_color.1));

    f.render_stateful_widget(list, area, &mut emu_state.mem.list_state);
}