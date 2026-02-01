use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem};

use crate::state::{EmuMode, EmuState, Selected};
use crate::ui::component::Component;
use super::{ANTI_FLASH_WHITE, BERKELEY_BLUE, CALIFORNIA_GOLD};

const INSTRUCTION_TITLE: &str = "Instruction";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Instruction;

impl Component for Instruction {
    fn render(f: &mut Frame, area: Rect, emu: &mut EmuState) {
        let items: Vec<ListItem> = emu.mach_snap.ins.list.iter().enumerate()
            .map(|(i, (addr, ins))| {

            let breakpoint = if emu.breakpoint_set.contains(&i) {
                "•"
            } else {
                " "
            };

            if *addr == emu.mach_snap.pc && emu.mode != EmuMode::Observation {
                emu.mach_snap.ins.list_state.select(Some(i));
                emu.mach_snap.ins.current_select = i;
            }
            ListItem::new(format!("{}{}",breakpoint, ins))
        }).collect();

        let state = &mut emu.mach_snap.ins.list_state;
    
        let hl_color = if emu.selected == Selected::Ins && emu.mode == EmuMode::Observation {
            (ANTI_FLASH_WHITE, BERKELEY_BLUE)
        } else {
            (BERKELEY_BLUE, CALIFORNIA_GOLD)
        };
        
        let hl_symbol = if matches!(emu.mode, EmuMode::Running | EmuMode::Stay) {
            "PC>> "
        } else {
            ">>   "
        };

        let list = List::new(items)
            .block(Block::bordered().title(INSTRUCTION_TITLE))
            .style(Style::default().bg(BERKELEY_BLUE).fg(CALIFORNIA_GOLD))
            .highlight_style(Style::default().bg(hl_color.0).fg(hl_color.1))
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_symbol(hl_symbol)
            .scroll_padding(10);

        f.render_stateful_widget(list, area, state);
    }
}
