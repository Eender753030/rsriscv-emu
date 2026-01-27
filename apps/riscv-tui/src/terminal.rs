use crossterm::{
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};

use ratatui::{
    Terminal, 
    backend::CrosstermBackend,
    Frame,
};

use std::io;

use crate::state::EmuState;

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

    pub fn draw(&mut self, ui: fn(&mut Frame, &mut EmuState), emu_state: &mut EmuState) -> io::Result<()> {
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
