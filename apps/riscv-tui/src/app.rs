use std::time::Duration;

use anyhow::Result;

use riscv_core::RiscV;
use riscv_core::constance::DRAM_BASE_ADDR;
use riscv_disasm::disasm;
use riscv_loader::LoadInfo;

use crate::event::{self, EmuEvent};
use crate::state::{EmuMode, EmuState};
use crate::terminal::EmuTerminal;
use crate::ui::render;
use crate::event::key::KeyControl;

#[derive(Debug, PartialEq)]
pub struct EmuApp {
    machine: RiscV,
    info: LoadInfo,
    state: EmuState,
    should_quit: bool,
}

impl EmuApp {
    pub fn new(info: LoadInfo) -> Result<Self> {
        let mut machine = RiscV::default();
        machine.load_info(&info)?;

        let ins_list = disasm::disassembler(&info);

        let ins_len: usize = info.code.iter()
            .map(|(code, _)| code.len() / 4).sum();

        let state = EmuState::new(&mut machine, ins_len, ins_list);

        Ok(EmuApp { machine, info, state, should_quit: false })
    }

    pub fn run(&mut self) -> Result<()> { 
        let mut t = EmuTerminal::new()?;
    
        while !self.should_quit {
            t.draw(render::ui, &mut self.state)?;
            
            self.event()?;

            if self.state.mode == EmuMode::Running {
                self.running()?;
            }
        }
        Ok(())
    }

    fn running(&mut self) -> Result<()> {
        if (self.state.pc - DRAM_BASE_ADDR) as usize >= self.state.ins_len * 4{
            self.state.mode = EmuMode::Stay;
        } else {
            self.machine.step()?;

            self.state.update_data(&self.machine);
            self.state.running_mode_selected_update();
        }
        Ok(())
    }

    fn event(&mut self) -> Result<()> {
        let duration = Duration::from_millis(
            match self.state.mode {
                EmuMode::Running => 16,
                _ => 100,
            }
        );

        match event::poll_event(duration)? {
            EmuEvent::Key(key) => {
                match self.state.mode {
                    EmuMode::Observation => self.key_observation(key),
                    EmuMode::Stay => self.key_stay(key)?,
                    EmuMode::Running => self.key_running(key),
                }
            },
            EmuEvent::Resize(x, y) => {},
            EmuEvent::None => {},
        }

        
        Ok(())
    }

    fn key_observation(&mut self, key: KeyControl) {
        use KeyControl::*;
        match key {
            Quit => self.should_quit = true,
            ChangeMode => {
                self.state.running_mode_selected();
                self.state.mode = EmuMode::Stay;
            }
            GoNext => self.state.next(),
            GoPrev => self.state.prev(),
            GoLeft => self.state.go_left(),
            GoRight => self.state.go_right(),
            NextPage => self.state.next_page(&self.machine),
            PrevPage => self.state.prev_page(&self.machine),
            ChangeMid => self.state.change_mid(),
            _ => {},
        }
    }

    fn key_stay(&mut self, key: KeyControl) -> Result<()> {
        use KeyControl::*;
        match key {
            Quit => self.should_quit = true,
            ChangeMid => self.state.change_mid(),
            ChangeMode => {
                self.state.observation_mode_selected();
                self.state.mode = EmuMode::Observation;
            },
            Reset => {
                self.machine.reset();
                self.machine.load_info(&self.info)?;
                self.state.update_data(&self.machine);
                self.state.running_mode_selected_update();
            },
            Step => {
                self.machine.step()?;
                
                self.state.update_data(&self.machine);
                self.state.running_mode_selected_update();
            },
            RunToEnd => self.state.mode = EmuMode::Running,
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