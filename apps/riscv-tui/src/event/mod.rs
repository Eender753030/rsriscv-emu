pub mod key;

use std::thread;
use std::time::Duration;
use std::sync::mpsc::Sender;

use anyhow::Result;

use crossterm::event::{self, Event, KeyEvent};

use key::{get_normal_key, get_edit_key};
use key::{KeyControl, EditKeyControl, NormalKeyControl};

use crate::state::EmuMode;
use crate::input::InputMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmuEvent {
    Key(KeyControl),
    Resize(u16, u16),
    Tick,
}

pub fn spawn_event_thread(tx: Sender<EmuEvent>) {
    thread::spawn(move || -> Result<()> {
        let mut input_mode = InputMode::default();
        let mut emu_mode = EmuMode::default();
        loop {
            let timeout = Duration::from_millis(100);

            if event::poll(timeout)? {
                match event::read()? {
                    Event::Key(KeyEvent{code, ..}) => {
                        match input_mode {
                            InputMode::Normal => {
                                if let Some(key) = get_normal_key(code) {
                                    match key {
                                        NormalKeyControl::ChangeMode => {
                                            emu_mode.change_mode();
                                        },
                                        NormalKeyControl::SearchBus => {
                                            if emu_mode == EmuMode::Observation {
                                                input_mode.edit();
                                            }
                                        },
                                        _ => {},
                                    }
                                    tx.send(EmuEvent::Key(KeyControl::Normal(key)))?;
                                }  
                            },
                            InputMode::Editting => {
                                if let Some(key) = get_edit_key(code) {
                                    if matches!(key, EditKeyControl::Exit | EditKeyControl::Enter) {
                                        input_mode = InputMode::Normal;
                                    }
                                    tx.send(EmuEvent::Key(KeyControl::Edit(key)))?;
                                }  
                            },
                        }
                    },
                    Event::Resize(x, y) => tx.send(EmuEvent::Resize(x, y))?,
                    _ => {},
                }
            } else {
                tx.send(EmuEvent::Tick)?;
            }
        }
    });
}
