mod component;
mod popup;

pub mod terminal;

use ratatui::{
    Frame, 
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Style, Color},
    widgets::{Block, Paragraph},
};

use crate::state::{EmuState, EmuMode, Mid};

use component::*;
use popup::*;

const HEADER: &str = concat!("RsRisc-V Emulator v", env!("CARGO_PKG_VERSION"));
const OBSERVATION_HINT_MESSAGE: &str = "(Q) Leave  (TAB) Switch mode  (V) Bus Search  (↑/↓) Scroll  (←/→) Change panel";
const EMULATE_HINT_MESSAGE: &str = "(Q) Leave  (TAB) Change mode  (S) Single step  (P) Run to end / Stop  (R) Reset";

pub(crate) const ANTI_FLASH_WHITE: Color = Color::Rgb(242, 242, 242);
pub(crate) const BERKELEY_BLUE: Color = Color::Rgb(0, 50, 98);
pub(crate) const CALIFORNIA_GOLD: Color = Color::Rgb(253, 181, 21);

pub fn ui(f: &mut Frame, emu: &mut EmuState) {
    let main_layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ]).split(f.area());

    
    render_header(f, main_layout[0], emu);
    render_content(f, main_layout[1], emu);
    match emu.mode {
        EmuMode::Observation | EmuMode::BusPopup => render_paragraph(f, main_layout[2], OBSERVATION_HINT_MESSAGE),
        EmuMode::Stay | EmuMode::Running => render_paragraph(f, main_layout[2], EMULATE_HINT_MESSAGE),
    }

    if emu.show_info_popup {
        info::render_popup(f, emu);
    }
    if emu.show_search_popup {
        search::render_popup(f, emu);
    } 
    if emu.show_bus_popup {
        bus::render_popup(f, emu);
    } 
}

fn render_header(f: &mut Frame, area: Rect, emu: &EmuState) {
    let message = match emu.mode {
        EmuMode::Observation | EmuMode::BusPopup => "Observation Mode",
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
