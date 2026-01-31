mod list_state;

use riscv_core::Exception;
use riscv_core::constance::DRAM_BASE_ADDR;
use riscv_core::debug::{DebugInterface, MachineInfo};

use list_state::ListStateRecord;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mid {
    #[default]
    Reg,
    #[cfg(feature = "zicsr")] Csr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Selected {
    #[default]
    Ins,
    Mid(Mid),
    Mem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EmuMode {
    #[default]
    Observation,
    Stay,
    Running,
}

#[derive(Debug, PartialEq, Eq)]
pub struct EmuState {
    pub ins: ListStateRecord<(u32, String)>,
    pub reg: ListStateRecord<u32>,
    #[cfg(feature = "zicsr")]
    pub csr: ListStateRecord<(String, u32)>,
    pub mem: ListStateRecord<u8>,
    pub pc: u32,
    pub except: String,

    pub mode: EmuMode,
    pub selected: Selected,
    pub mid_selected: Mid,
    pub page_selected: usize,
    pub ins_len: usize,

    machine_info: MachineInfo,
}

impl EmuState {
    pub fn new<D: DebugInterface>(machine: &D, ins_len: usize, ins_list: Vec<(u32, String)>) -> Self {
        let machine_info = machine.get_info();
        let (_, dram_base, page_size) = machine_info.get_info();

        let mut ins = ListStateRecord::new(ins_list);
        let reg = ListStateRecord::new(machine.inspect_regs().into_iter().collect());
        #[cfg(feature = "zicsr")]
        let csr = ListStateRecord::new(machine.inspect_csrs());
        let mem = ListStateRecord::new(machine.inspect_mem(dram_base, page_size));
        let except = "".to_string();
        let pc = machine.inspect_pc();

        let page_selected = 0;
        let mode = EmuMode::default();
        let selected = Selected::default();
        let mid_selected = Mid::default();
        
        ins.select_curr();

        EmuState { 
            #[cfg(feature = "zicsr")] csr,
            ins, reg, mem, except, pc, 
            mode, selected, mid_selected,
            page_selected, ins_len,
            machine_info,
        }
    }

    pub fn update_data<D: DebugInterface>(&mut self, machine: &D) {
        let (_, dram_base, page_size) = self.machine_info.get_info();
        let page_base = dram_base + (self.page_selected * page_size) as u32;

        self.reg.list = machine.inspect_regs().into_iter().collect();
        #[cfg(feature = "zicsr")] {
        self.csr.list = machine.inspect_csrs();
        }
        self.mem.list = machine.inspect_mem(page_base, page_size);
        self.pc = machine.inspect_pc();
    }

    pub fn update_exception(&mut self, except: Exception) {
        self.except = except.to_string()
    }

    pub fn running_mode_selected(&mut self) {
        match self.selected {
            Selected::Ins => {
                if (DRAM_BASE_ADDR..DRAM_BASE_ADDR + (self.ins_len * 4) as u32).contains(&self.pc) {
                    self.ins.select((self.pc - DRAM_BASE_ADDR) as usize);
                } else {
                    self.ins.no_select();
                }
            }
            Selected::Mid(m) => match m {
                Mid::Reg => self.reg.no_select(),
                #[cfg(feature = "zicsr")]
                Mid::Csr => self.csr.no_select(),
            }
            Selected::Mem => self.mem.no_select(),
        }
    }

    pub fn observation_mode_selected(&mut self) {
        match self.selected {
            Selected::Ins => self.ins.select_curr(),
            Selected::Mid(m) => match m {
                Mid::Reg => self.reg.select_curr(),
                #[cfg(feature = "zicsr")]
                Mid::Csr => self.csr.select_curr(),
            }
            Selected::Mem => self.mem.select_curr(),
        }
    }

    pub fn go_left(&mut self) {  
        self.selected = match self.selected {
            Selected::Ins => {
                self.ins.no_select();
                self.mem.select_curr();
                Selected::Mem
            },
            Selected::Mid(mid) => {
                match mid {
                    Mid::Reg => self.reg.no_select(),
                    #[cfg(feature = "zicsr")]
                    Mid::Csr => self.csr.no_select(),
                }
                self.ins.select_curr();
                Selected::Ins
            },
            Selected::Mem => {
                self.mem.no_select();
                match self.mid_selected {
                    Mid::Reg => self.reg.select_curr(),
                    #[cfg(feature = "zicsr")]
                    Mid::Csr => self.csr.select_curr(),
                }
                Selected::Mid(self.mid_selected)
            },
        };
    }

    pub fn go_right(&mut self) {  
        self.selected = match self.selected {
            Selected::Ins => {
                self.ins.no_select();
                match self.mid_selected {
                    Mid::Reg => self.reg.select_curr(),
                    #[cfg(feature = "zicsr")]
                    Mid::Csr => self.csr.select_curr(),
                }
                Selected::Mid(self.mid_selected)
            },
            Selected::Mid(mid) => {
                match mid {
                    Mid::Reg => self.reg.no_select(),
                    #[cfg(feature = "zicsr")]
                    Mid::Csr => self.csr.no_select(),
                }
                self.mem.select_curr();
                Selected::Mem
            },
            Selected::Mem => {
                self.mem.no_select();
                self.ins.select_curr();
                Selected::Ins
            },
        };
    }

    pub fn next(&mut self) {  
        match self.selected {
            Selected::Ins => self.ins.next(self.ins.list.len()),
            Selected::Mid(m) => match m {
                Mid::Reg => self.reg.next(self.reg.list.len()),
                #[cfg(feature = "zicsr")]
                Mid::Csr => self.csr.next(self.csr.list.len()),
            },
            Selected::Mem => self.mem.next(self.mem.list.len() / 4),
        }
    }
    
    pub fn prev(&mut self) {
        match self.selected {
            Selected::Ins => self.ins.prev(self.ins.list.len()),
            Selected::Mid(m) => match m {
                Mid::Reg => self.reg.prev(self.reg.list.len()),
                #[cfg(feature = "zicsr")]
                Mid::Csr => self.csr.prev(self.csr.list.len()),
            },
            Selected::Mem => self.mem.prev(self.mem.list.len() / 4),
        }
    }

    pub fn prev_page<D: DebugInterface>(&mut self, machine: &D) {
        let (_, dram_base, page_size) = self.machine_info.get_info();

        if self.page_selected != 0 {
            self.page_selected -= 1;
            let page_base = dram_base + (self.page_selected * page_size) as u32;
            self.mem.list = machine.inspect_mem(page_base, page_size);
        }
    }

    pub fn next_page<D: DebugInterface>(&mut self, machine: &D) {
        let (dram_size, dram_base, page_size) = self.machine_info.get_info();
    
        if self.page_selected < dram_size / page_size {
            self.page_selected += 1;
            let page_base = dram_base + (self.page_selected * page_size) as u32;
            self.mem.list = machine.inspect_mem(page_base, page_size);
        }
    }

    #[cfg(feature = "zicsr")]
    pub fn change_mid(&mut self) {
        match self.mid_selected {
            Mid::Reg => self.mid_selected = Mid::Csr,
            Mid::Csr => self.mid_selected = Mid::Reg,
        }
    }
}