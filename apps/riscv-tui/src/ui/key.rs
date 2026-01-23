use crossterm::event::{self, Event, KeyEvent, KeyCode};

use std::time::Duration;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyControl {
    Quit,
    GoNext,
    GoPrev,
    GoLeft,
    GoRight,
    NextPage,
    PrevPage,
    ChangeMid,
    ChangeMode,
    Reset,
    Step,
    RunToEnd,
    None,
}

pub fn poll_key_event(timeout: Duration) -> io::Result<KeyControl> {
    if event::poll(timeout)? && let Event::Key(KeyEvent{code, ..}) = event::read()? {
        return Ok(match code {
            KeyCode::Char('q' | 'Q') => KeyControl::Quit,
            KeyCode::Char('r' | 'R') => KeyControl::Reset,
            KeyCode::Char('s' | 'S') => KeyControl::Step,
            KeyCode::Char('p' | 'P') => KeyControl::RunToEnd,
            KeyCode::Char('c' | 'C') => KeyControl::ChangeMid,
            KeyCode::Char(']') => KeyControl::NextPage,
            KeyCode::Char('[') => KeyControl::PrevPage,
            KeyCode::Up => KeyControl::GoPrev,
            KeyCode::Down => KeyControl::GoNext,
            KeyCode::Left => KeyControl::GoLeft,
            KeyCode::Right => KeyControl::GoRight,
            KeyCode::Tab => KeyControl::ChangeMode,
            _ => KeyControl::None
        })
    }        
    Ok(KeyControl::None)
}