use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::ListItem;

use riscv_core::constance::PAGE_SIZE;
use riscv_core::debug::DebugInterface;

use crate::ui::component::Componet;
use crate::ui::state::EmuState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Memory;

impl Componet for Memory {
    fn render<D: DebugInterface>(f: &mut Frame, area: Rect, emu: &mut EmuState<D>) {
        let items: Vec<ListItem> = emu.mem.list.chunks(4).enumerate()
            .map(|(i, data)| {
                ListItem::new(format!(" {:#010x}: {}", i * 16 + (emu.page_selected * PAGE_SIZE), 
                    data.iter().map(|d| format!("{:02x} ", d)).collect::<String>()))
            }).collect();

        let title = format!(
            "Dram [{:#010x} - {:#010x}]", 
            emu.page_selected * PAGE_SIZE, 
            (emu.page_selected + 1) * PAGE_SIZE - 1
        );
        let state = &mut emu.mem.list_state;

        Self::list_state_render(f, area, items, state, &title);
    }
}
