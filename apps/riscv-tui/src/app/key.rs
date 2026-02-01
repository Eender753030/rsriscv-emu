
use anyhow::Result;

use crate::EmuApp;
use crate::event::key::KeyControl; 
use crate::state::EmuMode;

impl EmuApp {
    pub(super) fn key_observation(&mut self, key: KeyControl) {
        use KeyControl::*;
        match key {
            Quit => self.should_quit = true,
            ChangeMode  => self.state.mode = EmuMode::Stay,
            GoNext      => self.state.next(),
            GoPrev      => self.state.prev(),
            ChangePanel => self.state.change_panel(),
            #[cfg(feature = "zicsr")]
            ChangeMid   => self.state.change_mid(),
            ChangeView  => self.state.change_view(),
            BreakPoint  => self.state.breakpoint(),
            _ => {},
        }
    }

    pub(super) fn key_stay(&mut self, key: KeyControl) -> Result<()> {
        use KeyControl::*;
        match key {
            Quit => self.should_quit = true,
            #[cfg(feature = "zicsr")]
            ChangeMid => self.state.change_mid(),
            ChangeView  => self.state.change_view(),
            ChangeMode => {
                self.state.observation_mode_selected();
                self.state.mode = EmuMode::Observation;
            },
            Reset => {
                self.mach.reset();
                self.mach.load_info(&self.info)?;
                self.state.mach_snap.update_snapshot(&self.mach);
                self.state.mach_snap.reset_exception();
            },
            Step     => {
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
                self.state.mode = EmuMode::Running;
            }
            _ => {},
        }
        Ok(())
    }

    pub(super) fn key_running(&mut self, key: KeyControl) {
        use KeyControl::*;
        match key {
            Quit => self.should_quit = true,
            RunToEnd => self.state.mode = EmuMode::Stay,
            ChangeMode => {
                self.state.observation_mode_selected();
                self.state.mode = EmuMode::Observation;
            },
            _ => {},
        }
    }
}