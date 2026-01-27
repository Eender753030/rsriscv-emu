use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::ListItem;

use crate::ui::component::Component;
use crate::state::EmuState;

use super::MID_TITLE;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Csr;

impl Component for Csr {
    fn render(f: &mut Frame, area: Rect, emu: &mut EmuState) {
        let items: Vec<ListItem> = emu.csr.list.iter()
            .map(|(name, data)| {
                    ListItem::new(format!(" {:<7}: {}", name, data))
            }).collect();

        let state = &mut emu.csr.list_state;
        Self::list_state_render(f, area, items, state, MID_TITLE);
    }
}