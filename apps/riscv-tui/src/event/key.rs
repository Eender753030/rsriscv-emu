use crossterm::event::KeyCode;

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
}

pub fn poll_key_event(keycode: KeyCode) -> Option<KeyControl> {
    Some(match keycode {
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
        _                        => return None,
    })
}
