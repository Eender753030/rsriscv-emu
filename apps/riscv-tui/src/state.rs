mod list_state;

use std::collections::HashSet;

use riscv_core::Exception;
use riscv_core::debug::DebugInterface;

use list_state::ListStateRecord;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mid {
    #[default]
    Reg,
    #[cfg(feature = "zicsr")] Csr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DataView {
    #[default]
    Decimal,
    Hex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Selected {
    #[default]
    Ins,
    Mid(Mid),
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
    pub pc: u32,
    pub except: String,

    pub mode: EmuMode,
    pub selected: Selected,
    pub mid_selected: Mid,

    pub data_view: DataView,

    pub breakpoint_set: HashSet<usize>,
}

impl EmuState {
    pub fn new<D: DebugInterface>(machine: &D, ins_list: Vec<(u32, String)>) -> Self {
        let ins = ListStateRecord::new(ins_list);
        let reg = ListStateRecord::new(machine.inspect_regs().into_iter().collect());
        #[cfg(feature = "zicsr")]
        let csr = ListStateRecord::new(machine.inspect_csrs());

        let except = "".to_string();
        let pc = machine.inspect_pc();

        let mode = EmuMode::default();
        let selected = Selected::default();
        let mid_selected = Mid::default();
        let data_view = DataView::default();

        let breakpoint_set = HashSet::new();

        EmuState { 
            #[cfg(feature = "zicsr")] csr,
            ins, reg, except, pc, 
            mode, selected, mid_selected, data_view,
            breakpoint_set
        }
    }

    pub fn update_data<D: DebugInterface>(&mut self, machine: &D) {
        self.reg.list = machine.inspect_regs().into_iter().collect();
        #[cfg(feature = "zicsr")] {
        self.csr.list = machine.inspect_csrs();
        }
        self.pc = machine.inspect_pc();
    }

    pub fn update_exception(&mut self, except: Exception) {
        self.except = except.to_string()
    }

    pub fn observation_mode_selected(&mut self) {
        self.ins.select_curr()
    }

    pub fn change_panel(&mut self) {  
        self.selected = match self.selected {
            Selected::Ins    => Selected::Mid(self.mid_selected),
            Selected::Mid(_) => Selected::Ins,
        }
    }
    
    pub fn next(&mut self) {  
        match self.selected {
            Selected::Ins => self.ins.next(self.ins.list.len()),
            Selected::Mid(m) => match m {
                Mid::Reg => self.reg.next(self.reg.list.len()),
                #[cfg(feature = "zicsr")]
                Mid::Csr => self.csr.next(self.csr.list.len()),
            },
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
        }
    }

    #[cfg(feature = "zicsr")]
    pub fn change_mid(&mut self) {
        self.mid_selected = match self.mid_selected {
            Mid::Reg => Mid::Csr,
            Mid::Csr => Mid::Reg,
        };
        if matches!(self.selected, Selected::Mid(_)) {
            self.selected = Selected::Mid(self.mid_selected)
        } 
    }

    pub fn change_view(&mut self) {
        self.data_view = match self.data_view {
            DataView::Decimal => DataView::Hex,
            DataView::Hex     => DataView::Decimal,
        };
    }


    pub fn breakpoint(&mut self) {
        if !self.breakpoint_set.remove(&self.ins.current_select) {
            self.breakpoint_set.insert(self.ins.current_select);
        }
    }
}