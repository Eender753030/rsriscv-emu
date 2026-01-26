use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::ListItem;

use riscv_core::constance::DRAM_BASE_ADDR;
use riscv_core::debug::DebugInterface;

use crate::ui::component::Componet;
use crate::ui::state::{EmuMode, EmuState};

const INSTRUCTION_TITLE: &str = "Instruction";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Instruction;

impl Componet for Instruction {
    fn render<D: DebugInterface>(f: &mut Frame, area: Rect, emu: &mut EmuState<D>) {
        let mut offset = 0;

        let items: Vec<ListItem> = emu.ins.list.iter().enumerate()
            .map(|(i, ins)| {
            let marker = if ins.ends_with(':') {
                offset += 1;
                ""
            } else if (emu.pc - DRAM_BASE_ADDR) / 4 == ((i - offset) as u32) {
                if emu.mode != EmuMode::Observation {
                    emu.ins.list_state.select(Some(i));
                }
                "PC >>"
            } else {
                "     "
            };
            ListItem::new(format!("{}{}", marker, ins))
        }).collect();

        let state = &mut emu.ins.list_state;
        Self::list_state_render(f, area, items, state, INSTRUCTION_TITLE);
    }
}