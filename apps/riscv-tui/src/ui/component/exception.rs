use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Paragraph};

use crate::state::EmuState;
use crate::ui::component::Component;
use super::{BERKELEY_BLUE, CALIFORNIA_GOLD};

const EXCEPTION_TITLE: &str = "Code: Exception";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Exception;

impl Component for Exception {
    fn render(f: &mut Frame, area: Rect, emu: &mut EmuState) {
        let paragraph = Paragraph::new(emu.mach_snap.except.to_string())
            .block(Block::bordered().title(EXCEPTION_TITLE))
            .style(Style::default().bg(BERKELEY_BLUE).fg(CALIFORNIA_GOLD));
        
        f.render_widget(paragraph, area);
    }
}