use ratatui::{
    Frame,
    layout::{Layout, Rect, Constraint, Alignment},
    widgets::{Block, Paragraph},
    style::Style,  
};

use crate::state::{EmuState, EmuMode, Mid};
use super::component::*;

const HEADER: &str = concat!("RsRisc-V Emulator v", env!("CARGO_PKG_VERSION"));
const OBSERVATION_HINT_MESSAGE: &str = "(Q) Leave    (TAB) Switch mode    (↑/↓) Scroll    (←/→) Change panel";
const EMULATE_HINT_MESSAGE: &str = "(Q) Leave   (TAB) Change mode    (S) Single step    (P) Run to end / Stop    (R) Reset";

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
        EmuMode::Stay | EmuMode::Running => render_paragraph(f, main_layout[2], EMULATE_HINT_MESSAGE),
    }
}

fn render_header(f: &mut Frame, area: Rect, emu_state: &EmuState) {
    let message = match emu_state.mode {
        EmuMode::Observation => "Observation Mode",
        EmuMode::Stay | EmuMode::Running => "Emulate Mode"
    };
    
    let paragraph = Paragraph::new(message)
        .block(Block::bordered().title(HEADER))
        .style(Style::default().bg(BERKELEY_BLUE).fg(CALIFORNIA_GOLD))
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

fn render_paragraph(f: &mut Frame, area: Rect, message: &str) {
    let paragraph = Paragraph::new(message)
        .block(Block::bordered().title("Control Hint").title_alignment(Alignment::Center))
        .style(Style::default().bg(BERKELEY_BLUE).fg(CALIFORNIA_GOLD))
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

fn render_content(f: &mut Frame, area: Rect, emu: &mut EmuState) {
    let layout = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(3),
    ]).split(area);
    
    let info_layout = Layout::horizontal([
        Constraint::Percentage(70),
        Constraint::Percentage(30),
    ]).split(layout[0]);

    Instruction::render(f, info_layout[0], emu);
    match emu.mid_selected {
        Mid::Reg => Register::render(f, info_layout[1], emu),
        #[cfg(feature = "zicsr")]
        Mid::Csr => Csr::render(f, info_layout[1], emu),
    }
    Exception::render(f, layout[1], emu);
}
