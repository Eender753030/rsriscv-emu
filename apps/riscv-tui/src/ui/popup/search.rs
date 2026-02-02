use crate::{state::EmuState, ui::{ANTI_FLASH_WHITE, CALIFORNIA_GOLD}};

use ratatui::{
    Frame, 
    layout::{Alignment, Constraint, Flex, Layout, Position, Rect},
    style::{Color, Style},
    widgets::{Block, Clear, Paragraph},
};

pub fn render_popup(f: &mut Frame, emu: &EmuState) {
    let paragraph = Paragraph::new(emu.input.chars.as_str())
        .block(Block::bordered().style(Style::default().fg(CALIFORNIA_GOLD)).title("Enter Bus Address"))
        .style(Style::default().bg(Color::Rgb(20, 20, 20)).fg(ANTI_FLASH_WHITE))
        .alignment(Alignment::Left);
    
    let area = popup_area(f.area());

    f.render_widget(Clear, area); 
    f.render_widget(paragraph, area);

    f.set_cursor_position(Position::new(
        area.x + emu.input.cursor as u16 + 1, 
        area.y + 1,
    ));
}

fn popup_area(area: Rect) -> Rect {
    let vertical = Layout::vertical([Constraint::Length(3)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Length(25)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
