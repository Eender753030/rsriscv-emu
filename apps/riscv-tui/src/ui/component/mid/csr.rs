use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::ListItem;

use riscv_core::debug::DebugInterface;

use crate::ui::component::Componet;
use crate::ui::state::EmuState;

use super::MID_TITLE;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Csr;

impl Componet for Csr {
    fn render<D: DebugInterface>(f: &mut Frame, area: Rect, emu: &mut EmuState<D>) {
        let items: Vec<ListItem> = emu.csr.list.iter()
            .map(|(name, data)| {
                    ListItem::new(format!(" {:<7}: {}", name, data))
            }).collect();

        let state = &mut emu.csr.list_state;
        Self::list_state_render(f, area, items, state, MID_TITLE);
    }
}