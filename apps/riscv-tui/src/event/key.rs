use crossterm::event::KeyCode;

use KeyControl::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyControl {
    Quit,
    GoNext,
    GoPrev,
    ChangePanel,
    #[cfg(feature = "zicsr")] ChangeMid,
    ChangeMode,
    Reset,
    Step,
    RunToEnd,
    BreakPoint,
    SearchMem,
    ChangeView,
}

use KeyCode::*;

pub fn poll_key_event(keycode: KeyCode) -> Option<KeyControl> {
    Some(match keycode {
        Char('q' | 'Q') => Quit,
        Char('r' | 'R') => Reset,
        Char('s' | 'S') => Step,
        Char('p' | 'P') => RunToEnd,
        #[cfg(feature = "zicsr")]
        Char('c' | 'C') => ChangeMid,
        Char('b' | 'B') => BreakPoint,
        Char('m' | 'M') => SearchMem,
        Char('h' | 'H') => ChangeView,
        Up              => GoPrev,
        Down            => GoNext,
        Left | Right    => ChangePanel,
        Tab             => ChangeMode,
        _               => return None,
    })
}
