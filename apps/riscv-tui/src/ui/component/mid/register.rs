use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::ListItem;

use riscv_core::debug::DebugInterface;

use crate::ui::component::Componet;
use crate::ui::state::EmuState;

use super::MID_TITLE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Register;

impl Componet for Register {
    fn render<D: DebugInterface>(f: &mut Frame, area: Rect, emu: &mut EmuState<D>) {
        let items: Vec<ListItem> = emu.reg.list.iter().enumerate()
            .map(|(i, data)| {
                ListItem::new(format!(" x{:<2}: {}", i, data))
            }).collect();
    
        let state = &mut emu.reg.list_state;
        Self::list_state_render(f, area, items, state, MID_TITLE);
    }
}