use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem};

use crate::state::{DataView, EmuState, Selected};
use crate::ui::{ANTI_FLASH_WHITE, BERKELEY_BLUE, CALIFORNIA_GOLD};
use crate::ui::component::Component;

use super::MID_TITLE;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Csr;

impl Component for Csr {
    fn render(f: &mut Frame, area: Rect, emu: &mut EmuState) {
        let items: Vec<ListItem> = emu.mach_snap.csr.list.iter()
            .map(|(name, data)| {   
                match emu.data_view {
                    DataView::Decimal => ListItem::new(format!("{:<7}: {}", name, data)),
                    DataView::Hex     => ListItem::new(format!("{:<7}: {:#x}", name, data)),
                }
            }).collect();

        let state = &mut emu.mach_snap.csr.list_state;
        let hl_color = if matches!(emu.selected, Selected::Mid(_)) {
            (ANTI_FLASH_WHITE, BERKELEY_BLUE)
        } else {
            (BERKELEY_BLUE, CALIFORNIA_GOLD)
        };

        let list = List::new(items)
            .block(Block::bordered().title(MID_TITLE))
            .style(Style::default().bg(BERKELEY_BLUE).fg(CALIFORNIA_GOLD))
            .highlight_style(Style::default().bg(hl_color.0).fg(hl_color.1))
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_symbol(">> ")
            .scroll_padding(5);

        f.render_stateful_widget(list, area, state);
    }
}