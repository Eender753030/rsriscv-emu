use crossterm::{
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};

use ratatui::{
    Terminal, 
    backend::CrosstermBackend,
    Frame,
};
use riscv_core::debug::DebugInterface;

use std::io;

use crate::ui::state::EmuState;

type TerminalCross = Terminal<CrosstermBackend<io::Stdout>>;

pub struct EmuTerminal {
    terminal: TerminalCross
}

impl EmuTerminal {
    pub fn new() -> io::Result<EmuTerminal> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        Ok(EmuTerminal {
            terminal: Terminal::new(CrosstermBackend::new(io::stdout()))?
        })
    }

    pub fn draw<D: DebugInterface>(&mut self, ui: fn(&mut Frame, &mut EmuState<D>), emu_state: &mut EmuState<D>) -> io::Result<()> {
        self.terminal.draw(|f| ui(f, emu_state))?;
        Ok(())
    }
}

impl Drop for EmuTerminal {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}
