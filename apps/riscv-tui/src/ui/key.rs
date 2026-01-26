use std::time::Duration;
use std::io;

use crossterm::event::{self, Event, KeyEvent, KeyCode};

use KeyControl::*;

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
    Ok(if event::poll(timeout)? && let Event::Key(KeyEvent{code, ..}) = event::read()? {
        match code {
            KeyCode::Char('q' | 'Q') => Quit,
            KeyCode::Char('r' | 'R') => Reset,
            KeyCode::Char('s' | 'S') => Step,
            KeyCode::Char('p' | 'P') => RunToEnd,
            KeyCode::Char('c' | 'C') => ChangeMid,
            KeyCode::Char(']')       => NextPage,
            KeyCode::Char('[')       => PrevPage,
            KeyCode::Up              => GoPrev,
            KeyCode::Down            => GoNext,
            KeyCode::Left            => GoLeft,
            KeyCode::Right           => GoRight,
            KeyCode::Tab             => ChangeMode,

            _                        => KeyControl::None
        }
    } else {
        KeyControl::None
    })
}