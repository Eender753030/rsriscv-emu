
use anyhow::Result;

use crate::EmuApp;
use crate::event::key::{NormalKeyControl, EditKeyControl};
use crate::state::Selected; 

impl EmuApp {
    pub(super) fn key_observation(&mut self, key: NormalKeyControl) {
        use NormalKeyControl::*;
        match key {
            Quit => self.should_quit = true,
            ChangeMode  => {
                self.state.mode.stay();
                self.state.selected = Selected::Mid(self.state.mid_selected);
            }
            GoNext      => self.state.next(),
            GoPrev      => self.state.prev(),
            ChangePanel => self.state.change_panel(),
            #[cfg(feature = "zicsr")]
            ChangeMid   => self.state.change_mid(),
            ChangeView  => self.state.change_view(),
            BreakPoint  => if self.state.selected == Selected::Ins {
                self.state.breakpoint()
            },
            SearchBus   => {
                self.state.show_search_popup = true;
                self.state.input.mode.edit();
            },
            ShowInfo    => self.state.show_info_popup = !self.state.show_info_popup,
            _ => {},
        }
    }

    pub(super) fn key_stay(&mut self, key: NormalKeyControl) -> Result<()> {
        use NormalKeyControl::*;
        match key {
            Quit => self.should_quit = true,
            #[cfg(feature = "zicsr")]
            ChangeMid  => self.state.change_mid(),
            ChangeView => self.state.change_view(),
            GoNext     => self.state.next(),
            GoPrev     => self.state.prev(),
            ShowInfo   => self.state.show_info_popup = !self.state.show_info_popup,
            ChangeMode => {
                self.state.observation_mode_selected();
                self.state.mode.observation();
            },
            Reset => {
                self.mach.reset();
                self.mach.load_info(&self.info)?;
                self.state.mach_snap.update_snapshot(&self.mach);
                self.state.mach_snap.reset_exception();
            },
            Step => {
                #[cfg(not(feature = "zicsr"))]
                if self.step().is_err() {
                }
                #[cfg(feature = "zicsr")]
                self.step()?
            }
            RunToEnd => {
                #[cfg(not(feature = "zicsr"))]
                if self.step().is_err() {
                }
                #[cfg(feature = "zicsr")]
                self.step()?;
                self.state.mode.run();
            }
            _ => {},
        }
        Ok(())
    }

    pub(super) fn key_running(&mut self, key: NormalKeyControl) {
        use NormalKeyControl::*;
        match key {
            Quit => self.should_quit = true,
            RunToEnd => self.state.mode.stay(),
            ChangeMode => {
                self.state.observation_mode_selected();
                self.state.mode.observation();
            },
            _ => {},
        }
    }

     pub(super) fn key_popup(&mut self, key: NormalKeyControl) {
        use NormalKeyControl::*;
        match key {
            Quit => self.should_quit = true,
            ClosePopup => {
                self.state.mode.observation();
                self.state.show_bus_popup = false;
                self.state.temp_bus_view = None;
            }
            _ => {},
        }
    }

    pub(super) fn key_editting(&mut self, key: EditKeyControl) {
        use EditKeyControl::*;
        match key {
            Enter => {
                self.receive_bus_address();
                self.state.input.mode.normal();
                self.state.show_search_popup = false;
            },
            Char(new_char) => self.state.input.enter_char(new_char),
            CursorLeft => self.state.input.move_cursor_left(),
            CursorRight => self.state.input.move_cursor_right(),
            Delete => self.state.input.delete_char(),
            Exit => {
                self.state.input.mode.normal();
                self.state.input.clear();
                self.state.show_search_popup = false;
            },
        }
    }
}