use crate::state::EmuState;
use crate::ui::{ANTI_FLASH_WHITE, CALIFORNIA_GOLD};

use ratatui::{
    Frame, 
    layout::{Alignment, Constraint, Flex, Layout, Rect}, 
    style::{Style, Color}, 
    widgets::{Block, Clear, List, ListItem},
};

pub fn render_popup(f: &mut Frame, emu: &EmuState) {
    let area;

    let items: Vec<ListItem> = if let Some(bus) = &emu.temp_bus_view {
        area = popup_area(f.area(), 16, 25);
        let start_addr = bus.0;
        
        bus.1.chunks_exact(4).enumerate()
            .map(|(i, raw)| 
                ListItem::new(format!("{:#010x}: {}", 
                    start_addr + (i * 4) as u32,
                    raw.iter().map(|byte| 
                            format!("{:02x} ", byte)
                        ).collect::<String>()
                ))
            ).collect()
    } else {
        area = popup_area(f.area(), 3, 17);

        vec![ListItem::new("Unvalid Address")]
    };

    let list = List::new(items)
        .block(Block::bordered()
            .style(Style::default().fg(CALIFORNIA_GOLD)).title_bottom("(Esc) Close").title("Bus View").title_alignment(Alignment::Center))
        .style(Style::default().bg(Color::Rgb(20, 20, 20)).fg(ANTI_FLASH_WHITE));
        
    
    f.render_widget(Clear, area); 
    f.render_widget(list, area);
}

fn popup_area(area: Rect, width: u16, height: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Length(width)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Length(height)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

