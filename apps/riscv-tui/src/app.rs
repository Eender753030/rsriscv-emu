mod key;

use std::sync::mpsc::{self, Receiver};

use anyhow::Result;

use riscv_core::RiscV;
#[cfg(not(feature = "zicsr"))]
use riscv_core::RiscVError;
use riscv_core::debug::DebugInterface;
use riscv_disasm::disasm;
use riscv_loader::LoadInfo;

use crate::event::{self, EmuEvent};
use crate::event::key::KeyControl;
use crate::state::{EmuMode, EmuState};
use crate::ui;
use crate::ui::terminal::EmuTerminal;

#[derive(Debug)]
pub struct EmuApp {
    mach: RiscV,
    info: LoadInfo,
    state: EmuState,
    should_quit: bool,
    event_rx: Receiver<EmuEvent>,
}

impl EmuApp {
    pub fn new(info: LoadInfo) -> Result<Self> {
        let mut mach = RiscV::default();
        mach.load_info(&info)?;

        let ins_list = disasm::disassembler(&info);
        let state = EmuState::new(&mach, ins_list);
        

        let (event_tx, event_rx) = mpsc::channel::<EmuEvent>();
        event::spawn_event_thread(event_tx);

        Ok(EmuApp { 
            mach, info, state, 
            should_quit: false, event_rx 
        })
    }

    pub fn run(&mut self) -> Result<()> { 
        let mut t = EmuTerminal::new()?;
    
        while !self.should_quit {
            t.draw(ui::ui, &mut self.state)?;
            
            self.event()?;    
        }
        Ok(())
    }

    fn step(&mut self) -> Result<()> {
        if let Some(except) = self.mach.step()? {
            self.state.mach_snap.update_exception(except);
            #[cfg(not(feature = "zicsr"))]
            return Err(anyhow::Error::new(RiscVError::Exception));
        }
        self.state.mach_snap.update_snapshot(&self.mach);
        Ok(())
    }

    fn event(&mut self) -> Result<()> {
        match self.event_rx.recv()? {
            EmuEvent::Key(key) => {
                match key {
                    KeyControl::Normal(key) => match self.state.mode {
                        EmuMode::Observation => self.key_observation(key),
                        EmuMode::Stay        => self.key_stay(key)?,
                        EmuMode::Running     => self.key_running(key),
                        EmuMode::BusPopup    => self.key_popup(key),
                    },
                    KeyControl::Edit(key) => self.key_editting(key),
                }
                
            },
            EmuEvent::Resize(_, _) => {},
            EmuEvent::Tick => {
                if self.state.mode == EmuMode::Running {
                    if self.state.breakpoint_set
                        .contains(&self.state.mach_snap.ins.current_select) {
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

    pub fn receive_bus_address(&mut self) {
        if let Some(addr) = self.state.input.submit() {
            self.state.temp_bus_view = 
            Some((addr, self.mach.inspect_bus(addr, 68)));
        }
        self.state.mode.popup();
        self.state.show_bus_popup = true;
    }
}