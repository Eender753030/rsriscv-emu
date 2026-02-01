use std::sync::mpsc::{self, Receiver};

use anyhow::Result;

use riscv_core::RiscV;
#[cfg(not(feature = "zicsr"))]
use riscv_core::RiscVError;
use riscv_disasm::disasm;
use riscv_loader::LoadInfo;

use crate::event::{self, EmuEvent};
use crate::state::{EmuMode, EmuState};
use crate::terminal::EmuTerminal;
use crate::ui::render;
use crate::event::key::KeyControl;

#[derive(Debug)]
pub struct EmuApp {
    machine: RiscV,
    info: LoadInfo,
    state: EmuState,
    should_quit: bool,
    event_rx: Receiver<EmuEvent>,
}

impl EmuApp {
    pub fn new(info: LoadInfo) -> Result<Self> {
        let mut machine = RiscV::default();
        machine.load_info(&info)?;

        let ins_list = disasm::disassembler(&info);

        let (event_tx, event_rx) = mpsc::channel::<EmuEvent>();

        event::spawn_event_thread(event_tx);

        let state = EmuState::new(&machine, ins_list);

        Ok(EmuApp { machine, info, state, should_quit: false, event_rx })
    }

    pub fn run(&mut self) -> Result<()> { 
        let mut t = EmuTerminal::new()?;
    
        while !self.should_quit {
            t.draw(render::ui, &mut self.state)?;
            
            self.event()?;    
        }
        Ok(())
    }

    fn step(&mut self) -> Result<()> {
        if let Some(except) = self.machine.step()? {
            self.state.update_exception(except);
            #[cfg(not(feature = "zicsr"))]
            return Err(anyhow::Error::new(RiscVError::Exception));
        }
        self.state.update_data(&self.machine);
        Ok(())
    }

    fn event(&mut self) -> Result<()> {
        match self.event_rx.recv()? {
            EmuEvent::Key(key) => {
                match self.state.mode {
                    EmuMode::Observation => self.key_observation(key),
                    EmuMode::Stay        => self.key_stay(key)?,
                    EmuMode::Running     => self.key_running(key),
                }
            },
            EmuEvent::Resize(_, _) => {},
            EmuEvent::Tick => {
                if self.state.mode == EmuMode::Running {
                    if self.state.breakpoint_set.contains(&self.state.ins.current_select) {
                        self.state.mode = EmuMode::Stay 
                    } else {
                        #[cfg(not(feature = "zicsr"))]
                        if self.step().is_err() {
                            self.state.mode = EmuMode::Stay 
                        }
                        #[cfg(feature = "zicsr")]
                        self.step()?;
                    }
                }
            }
        }
        Ok(())
    }

    fn key_observation(&mut self, key: KeyControl) {
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

    fn key_stay(&mut self, key: KeyControl) -> Result<()> {
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
                self.machine.reset();
                self.machine.load_info(&self.info)?;
                self.state.update_data(&self.machine);
                self.state.except = "".to_string();
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

    fn key_running(&mut self, key: KeyControl) {
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