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
        ListItem::new(format!(" Dram Base: {:#010x}", emu.mach_snap.info.dram_base)),
        ListItem::new(format!(" Dram Size: {} GB", emu.mach_snap.info.dram_size)),
        ListItem::new(format!(" Page Size: {} KB", emu.mach_snap.info.page_size)),
        #[cfg(feature = "s")]
        ListItem::new(format!(" TLB Hit Rate: {:.2} %", emu.mach_snap.info.hit_rate * 100.0)),
        #[cfg(feature = "s")]
        ListItem::new(format!(" Current Mode: {}", emu.mach_snap.info.curr_mode)),
        ListItem::new(format!(" Current PC: {:#010x}", emu.mach_snap.pc)),
    ];

    let list = List::new(items)
        .block(Block::bordered().style(Style::default().fg(CALIFORNIA_GOLD)).title("Machine Information"))
        .style(Style::default().bg(Color::Rgb(20, 20, 20)).fg(ANTI_FLASH_WHITE));

    let area = popup_area(f.area());
    f.render_widget(Clear, area); 
    f.render_widget(list, area);
}

fn popup_area(area: Rect) -> Rect {
    let vertical = Layout::vertical([Constraint::Length(8)]).flex(Flex::End);
    let horizontal = Layout::horizontal([Constraint::Length(30)]).flex(Flex::End);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
