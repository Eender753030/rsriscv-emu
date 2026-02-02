use crate::state::EmuState;
use crate::ui::{ANTI_FLASH_WHITE, CALIFORNIA_GOLD};

use ratatui::{
    Frame, 
    layout::{Constraint, Flex, Layout, Rect}, 
    style::{Style, Color}, 
    widgets::{Block, Clear, List, ListItem},
};

pub fn render_popup(f: &mut Frame, emu: &EmuState) {
    let items = vec![
        ListItem::new(format!(" Dram Base: {:#010x}", emu.mach_info.dram_base)),
        ListItem::new(format!(" Dram Size: {} GB", emu.mach_info.dram_size)),
        ListItem::new(format!(" Page Size: {} KB", emu.mach_info.page_size)),
        ListItem::new(format!("  Hit Rate: {} %", emu.mach_info.hit_rate)),
        ListItem::new(format!("Current PC: {:#010x}", emu.mach_snap.pc)),
    ];

    let list = List::new(items)
        .block(Block::bordered().style(Style::default().fg(CALIFORNIA_GOLD)).title("Machine Information"))
        .style(Style::default().bg(Color::Rgb(20, 20, 20)).fg(ANTI_FLASH_WHITE));

    let area = popup_area(f.area());
    f.render_widget(Clear, area); 
    f.render_widget(list, area);
}

fn popup_area(area: Rect) -> Rect {
    let vertical = Layout::vertical([Constraint::Length(7)]).flex(Flex::End);
    let horizontal = Layout::horizontal([Constraint::Length(25)]).flex(Flex::End);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
