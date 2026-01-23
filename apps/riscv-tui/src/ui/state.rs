use ratatui::widgets::ListState;

use riscv_core::{constance::DRAM_BASE_ADDR, debug::{DebugInterface, MachineInfo}};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mid {
    Reg,
    Csr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selected {
    Ins,
    Mid(Mid),
    Mem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmuMode {
    Observation,
    Stay,
    Running,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListStateRecord<T> {
    pub list: Vec<T>,
    pub list_state: ListState,
    current_select: usize,
}

impl <T> ListStateRecord<T> {
    pub fn new(list: Vec<T>) -> Self {
        ListStateRecord { 
            list,
            ..Default::default()
        }
    }
}

impl <T> Default for ListStateRecord<T> {
    fn default() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        ListStateRecord {
            list: Vec::new(),
            list_state, 
            current_select: 0, 
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct EmuState<'a, D: DebugInterface> {
    pub machine: &'a mut D,
    pub ins: ListStateRecord<String>,
    pub reg: ListStateRecord<u32>,
    pub csr: ListStateRecord<(String, u32)>,
    pub mem: ListStateRecord<u8>,
    pub pc: u32,
    pub mode: EmuMode,
    pub selected: Selected,
    pub mid_selected: Mid,
    pub page_selected: usize,
    code_len: usize,

    machine_info: MachineInfo,
}

impl <'a, D: DebugInterface> EmuState<'a, D> {
    pub fn new(machine: &'a mut D, code_len: usize, ins_list: Vec<String>) -> Self {
        let machine_info = machine.get_info();
        let (_, dram_base, page_size) = machine_info.get_info();

        let ins = ListStateRecord::new(ins_list);
        let reg = ListStateRecord::new(machine.inspect_regs().into_iter().collect());
        let csr = ListStateRecord::new(machine.inspect_csrs());
        let mem = ListStateRecord::new(machine.inspect_mem(dram_base, page_size));
        let pc = machine.inspect_pc();
        let page_selected = 0;
        let mode = EmuMode::Observation;
        let selected = Selected::Ins;
        let mid_selected = Mid::Reg;
        
        
        EmuState { 
            machine, 
            ins, reg, csr, mem, pc, 
            mode, selected, mid_selected,
            page_selected, code_len,
            machine_info,
        }
    }

    pub fn update_data(&mut self) {
        let (_, dram_base, page_size) = self.machine_info.get_info();
        let page_base = dram_base + (self.page_selected * page_size) as u32;

        self.reg.list = self.machine.inspect_regs().into_iter().collect();
        self.csr.list = self.machine.inspect_csrs();
        self.mem.list = self.machine.inspect_mem(page_base, page_size);
        self.pc = self.machine.inspect_pc();
    }

    pub fn running_mode_selected(&mut self) {
        match self.selected {
            Selected::Ins => self.ins.list_state.select(Some((self.pc - DRAM_BASE_ADDR) as usize)),
            Selected::Mid(m) => match m {
                Mid::Reg => self.reg.list_state.select(None),
                Mid::Csr => self.csr.list_state.select(None),
            }
            Selected::Mem => self.mem.list_state.select(None)
        }
    }

    pub fn running_mode_selected_update(&mut self) {
        self.ins.list_state.select(Some((self.pc - DRAM_BASE_ADDR) as usize / 4))
    }

    pub fn observation_mode_selected(&mut self) {
        match self.selected {
            Selected::Ins => self.ins.list_state.select(Some(self.ins.current_select)),
            Selected::Mid(m) => match m {
                Mid::Reg => self.reg.list_state.select(Some(self.reg.current_select)),
                Mid::Csr => self.csr.list_state.select(Some(self.csr.current_select)),
            }
            Selected::Mem => self.mem.list_state.select(Some(self.mem.current_select)),
        }
    }

    pub fn go_left(&mut self) {  
        self.selected = match self.selected {
            Selected::Ins => Selected::Mem,
            Selected::Mid(_) => Selected::Ins,
            Selected::Mem => Selected::Mid(self.mid_selected) ,
        };
    }

    pub fn go_right(&mut self) {  
        self.selected = match self.selected {
            Selected::Ins => Selected::Mid(self.mid_selected) ,
            Selected::Mid(_) => Selected::Mem, 
            Selected::Mem => Selected::Ins,
        };
    }

    pub fn next(&mut self) {  
        match self.selected {
            Selected::Ins => {
                self.ins.current_select = match self.ins.current_select >= self.ins.list.len() - 1 {
                    true => 0,
                    false => self.ins.current_select + 1
                };
                self.ins.list_state.select(Some(self.ins.current_select));
            },
            Selected::Mid(m) => {
                match m {
                    Mid::Reg => {
                        self.reg.current_select = match self.reg.current_select >= self.reg.list.len() - 1 {
                            true => 0,
                            false => self.reg.current_select + 1
                        };
                        self.reg.list_state.select(Some(self.reg.current_select));
                    },
                    Mid::Csr => {
                        self.csr.current_select = match self.csr.current_select >= self.csr.list.len() - 1 {
                            true => 0,
                            false => self.csr.current_select + 1
                        };
                        self.csr.list_state.select(Some(self.csr.current_select));
                    }
                }
                
            },
            Selected::Mem => {
                self.mem.current_select = match self.mem.current_select >= (self.mem.list.len() / 4) - 1 {
                    true => 0,
                    false => self.mem.current_select + 1
                };
                self.mem.list_state.select(Some(self.mem.current_select));
            },
        }
    }
    
    pub fn prev(&mut self) {
        match self.selected {
            Selected::Ins => {
                self.ins.current_select = match self.ins.current_select == 0 {
                    true => self.ins.list.len() - 1,
                    false => self.ins.current_select - 1
                };
                self.ins.list_state.select(Some(self.ins.current_select));
            },
            Selected::Mid(m) => {
                match m {
                    Mid::Reg => {
                        self.reg.current_select = match self.reg.current_select == 0  {
                            true => self.reg.list.len() - 1,
                            false => self.reg.current_select - 1
                        };
                        self.reg.list_state.select(Some(self.reg.current_select));
                    },
                    Mid::Csr => {
                        self.csr.current_select = match self.csr.current_select == 0  {
                            true => self.csr.list.len() - 1,
                            false => self.csr.current_select - 1
                        };
                        self.csr.list_state.select(Some(self.csr.current_select));
                    }
                }
                
            },
            Selected::Mem => {
                self.mem.current_select = match self.mem.current_select == 0  {
                    true => (self.mem.list.len() / 4) - 1,
                    false => self.mem.current_select - 1
                };
                self.mem.list_state.select(Some(self.mem.current_select));
            },
        }
    }

    pub fn prev_page(&mut self) {
        let (_, dram_base, page_size) = self.machine_info.get_info();

        if self.page_selected != 0 {
            self.page_selected -= 1;
            let page_base = dram_base + (self.page_selected * page_size) as u32;
            self.mem.list = self.machine.inspect_mem(page_base, page_size);
        }
    }

    pub fn next_page(&mut self) {
        let (dram_size, dram_base, page_size) = self.machine_info.get_info();
    
        if self.page_selected < dram_size / page_size {
            self.page_selected += 1;
            let page_base = dram_base + (self.page_selected * page_size) as u32;
            self.mem.list = self.machine.inspect_mem(page_base, page_size);
        }
    }

    pub fn change_mid(&mut self) {
        match self.mid_selected {
            Mid::Reg => self.mid_selected = Mid::Csr,
            Mid::Csr => self.mid_selected = Mid::Reg,
        }
    }
}