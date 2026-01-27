pub mod key;

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyEvent};
use key::KeyControl;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmuEvent {
    Key(KeyControl),
    Resize(u16, u16),
    None,
}

pub fn poll_event(timeout: Duration) -> io::Result<EmuEvent> {
    let event = if event::poll(timeout)? {
        match event::read()? {
            Event::Key(KeyEvent{code, ..}) => {
                match key::poll_key_event(code) {
                    Some(key) => EmuEvent::Key(key),
                    None      => EmuEvent::None,
                }
            }
            Event::Resize(x, y) => EmuEvent::Resize(x, y),
            _ => EmuEvent::None,
        }
    } else {
        EmuEvent::None
    };

    Ok(event)
}
