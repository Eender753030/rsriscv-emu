use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::ListItem;

use crate::ui::component::Component;
use crate::state::EmuState;

use super::MID_TITLE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Register;

impl Component for Register {
    fn render(f: &mut Frame, area: Rect, emu: &mut EmuState) {
        let items: Vec<ListItem> = emu.reg.list.iter().enumerate()
            .map(|(i, data)| {
                ListItem::new(format!(" x{:<2}: {}", i, data))
            }).collect();
    
        let state = &mut emu.reg.list_state;
        Self::list_state_render(f, area, items, state, MID_TITLE);
    }
}