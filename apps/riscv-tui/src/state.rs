mod list_state;

use riscv_core::constance::DRAM_BASE_ADDR;
use riscv_core::debug::{DebugInterface, MachineInfo};

use list_state::ListStateRecord;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mid {
    #[default]
    Reg,
    Csr,
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
    pub ins: ListStateRecord<String>,
    pub reg: ListStateRecord<u32>,
    pub csr: ListStateRecord<(String, u32)>,
    pub mem: ListStateRecord<u8>,
    pub pc: u32,
    pub mode: EmuMode,
    pub selected: Selected,
    pub mid_selected: Mid,
    pub page_selected: usize,
    pub ins_len: usize,

    machine_info: MachineInfo,
}

impl EmuState {
    pub fn new<D: DebugInterface>(machine: &D, ins_len: usize, ins_list: Vec<String>) -> Self {
        let machine_info = machine.get_info();
        let (_, dram_base, page_size) = machine_info.get_info();

        let mut ins = ListStateRecord::new(ins_list);
        let reg = ListStateRecord::new(machine.inspect_regs().into_iter().collect());
        let csr = ListStateRecord::new(machine.inspect_csrs());
        let mem = ListStateRecord::new(machine.inspect_mem(dram_base, page_size));
        let pc = machine.inspect_pc();

        let page_selected = 0;
        let mode = EmuMode::default();
        let selected = Selected::default();
        let mid_selected = Mid::default();
        
        ins.select_curr();

        EmuState { 
            ins, reg, csr, mem, pc, 
            mode, selected, mid_selected,
            page_selected, ins_len,
            machine_info,
        }
    }

    pub fn update_data<D: DebugInterface>(&mut self, machine: &D) {
        let (_, dram_base, page_size) = self.machine_info.get_info();
        let page_base = dram_base + (self.page_selected * page_size) as u32;

        self.reg.list = machine.inspect_regs().into_iter().collect();
        self.csr.list = machine.inspect_csrs();
        self.mem.list = machine.inspect_mem(page_base, page_size);
        self.pc = machine.inspect_pc();
    }

    pub fn running_mode_selected(&mut self) {
        match self.selected {
            Selected::Ins => self.ins.select((self.pc - DRAM_BASE_ADDR) as usize),
            Selected::Mid(m) => match m {
                Mid::Reg => self.reg.no_select(),
                Mid::Csr => self.csr.no_select(),
            }
            Selected::Mem => self.mem.no_select(),
        }
    }

    pub fn running_mode_selected_update(&mut self) {
        self.ins.select((self.pc - DRAM_BASE_ADDR) as usize / 4)
    }

    pub fn observation_mode_selected(&mut self) {
        match self.selected {
            Selected::Ins => self.ins.select_curr(),
            Selected::Mid(m) => match m {
                Mid::Reg => self.reg.select_curr(),
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
                    Mid::Csr => self.csr.no_select(),
                }
                self.ins.select_curr();
                Selected::Ins
            },
            Selected::Mem => {
                self.mem.no_select();
                match self.mid_selected {
                    Mid::Reg => self.reg.select_curr(),
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
                    Mid::Csr => self.csr.select_curr(),
                }
                Selected::Mid(self.mid_selected)
            },
            Selected::Mid(mid) => {
                match mid {
                    Mid::Reg => self.reg.no_select(),
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
            Selected::Ins => self.ins.next(),
            Selected::Mid(m) => match m {
                Mid::Reg => self.reg.next(),
                Mid::Csr => self.csr.next(),
            },
            Selected::Mem => self.mem.next(),
        }
    }
    
    pub fn prev(&mut self) {
        match self.selected {
            Selected::Ins => self.ins.prev(),
            Selected::Mid(m) => match m {
                Mid::Reg => self.reg.prev(),
                Mid::Csr => self.csr.prev(),
            },
            Selected::Mem => self.mem.prev(),
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

    pub fn change_mid(&mut self) {
        match self.mid_selected {
            Mid::Reg => self.mid_selected = Mid::Csr,
            Mid::Csr => self.mid_selected = Mid::Reg,
        }
    }
}