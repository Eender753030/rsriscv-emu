pub mod instruction;
pub mod memory;
pub mod mid;

use ratatui::{
    Frame, 
    layout::Rect,
    style::{Color, Style}, 
    widgets::{Block, List, ListItem, ListState},
};

use riscv_core::debug::DebugInterface;

use crate::ui::state::EmuState;

pub use instruction::Instruction;
pub use memory::Memory;
pub use mid::csr::Csr;
pub use mid::register::Register;

const ANTI_FLASH_WHITE: Color = Color::Rgb(242, 242, 242);
const BERKELEY_BLUE: Color = Color::Rgb(0, 50, 98);
const CALIFORNIA_GOLD: Color = Color::Rgb(253, 181, 21);

pub trait Componet {
    fn render<D: DebugInterface>(f: &mut Frame, area: Rect, emu: &mut EmuState<D>);

    fn list_state_render(
        f: &mut Frame, 
        area: Rect, 
        items: Vec<ListItem>,
        state: &mut ListState, 
        title: &str
    ) {
    let list = List::new(items)
        .block(Block::bordered().title(title))
        .style(Style::default().bg(BERKELEY_BLUE).fg(CALIFORNIA_GOLD))
        .highlight_style(Style::default().bg(ANTI_FLASH_WHITE).fg(BERKELEY_BLUE));

    f.render_stateful_widget(list, area, state);
}
}

