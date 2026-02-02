use crossterm::event::KeyCode;

use NormalKeyControl::*;
use EditKeyControl::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalKeyControl {
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
    SearchBus,
    ChangeView,
    ShowInfo,
    ClosePopup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditKeyControl {
    Enter,
    Delete,
    CursorLeft,
    CursorRight,
    Exit,
    Char(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyControl {
    Normal(NormalKeyControl),
    Edit(EditKeyControl),
}

pub fn get_normal_key(keycode: KeyCode) -> Option<NormalKeyControl> {
    Some(match keycode {
            KeyCode::Char('q' | 'Q') => Quit,
            KeyCode::Char('r' | 'R') => Reset,
            KeyCode::Char('s' | 'S') => Step,
            KeyCode::Char('p' | 'P') => RunToEnd,
            #[cfg(feature = "zicsr")]
            KeyCode::Char('c' | 'C') => ChangeMid,
            KeyCode::Char('b' | 'B') => BreakPoint,
            KeyCode::Char('v' | 'V') => SearchBus,
            KeyCode::Char('h' | 'H') => ChangeView,
            KeyCode::Char('i' | 'I') => ShowInfo,
            KeyCode::Up              => GoPrev,
            KeyCode::Down            => GoNext,
            KeyCode::Left            => ChangePanel,
            KeyCode::Right           => ChangePanel,
            KeyCode::Tab             => ChangeMode,
            KeyCode::Esc             => ClosePopup,
            _                        => return None,
    })
}

pub fn get_edit_key(keycode: KeyCode) -> Option<EditKeyControl> {
    Some(match keycode {
            KeyCode::Enter           => Enter,
            KeyCode::Char(to_insert) => Char(to_insert),
            KeyCode::Backspace       => Delete,
            KeyCode::Left            => CursorLeft,
            KeyCode::Right           => CursorRight,
            KeyCode::Esc             => Exit,
            _                        => return None,
    })
}