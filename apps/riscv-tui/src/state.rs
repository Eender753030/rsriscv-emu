mod list_state;
mod mode;
mod snapshot;

use std::collections::HashSet;

use riscv_core::debug::DebugInterface;

use snapshot::MachineSnapshot;

pub use mode::{DataView, EmuMode, Mid, Selected};

#[derive(Debug, PartialEq, Eq)]
pub struct EmuState {
    pub mach_snap: MachineSnapshot,

    pub mode: EmuMode,
    pub selected: Selected,
    pub mid_selected: Mid,
    pub data_view: DataView,

    pub breakpoint_set: HashSet<usize>,
}

impl EmuState {
    pub fn new<D: DebugInterface>(mach: &D, ins_list: Vec<(u32, String)>) -> Self {
        let mach_snap = MachineSnapshot::new(mach, ins_list);

        let mode = EmuMode::default();
        let selected = Selected::default();
        let mid_selected = Mid::default();
        let data_view = DataView::default();

        let breakpoint_set = HashSet::new();

        EmuState { 
            mach_snap,
            mode, selected, mid_selected, data_view,
            breakpoint_set
        }
    }

    pub fn observation_mode_selected(&mut self) {
        self.mach_snap.ins.select_curr()
    }

    pub fn change_panel(&mut self) {  
        self.selected = match self.selected {
            Selected::Ins    => Selected::Mid(self.mid_selected),
            Selected::Mid(_) => Selected::Ins,
        }
    }
    
    pub fn next(&mut self) {  
        match self.selected {
            Selected::Ins => self.mach_snap.ins
                .next(self.mach_snap.ins.list.len()),

            Selected::Mid(m) => match m {
                Mid::Reg => self.mach_snap.reg
                    .next(self.mach_snap.reg.list.len()),

                #[cfg(feature = "zicsr")]
                Mid::Csr => self.mach_snap.csr.
                    next(self.mach_snap.csr.list.len()),
            },
        }
    }
    
    pub fn prev(&mut self) {
        match self.selected {
            Selected::Ins => self.mach_snap.ins
                .prev(self.mach_snap.ins.list.len()),

            Selected::Mid(m) => match m {
                Mid::Reg => self.mach_snap.reg
                    .prev(self.mach_snap.reg.list.len()),
                    
                #[cfg(feature = "zicsr")]
                Mid::Csr => self.mach_snap.csr
                    .prev(self.mach_snap.csr.list.len()),
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
        if !self.breakpoint_set
            .remove(&self.mach_snap.ins.current_select) {
            self.breakpoint_set
                .insert(self.mach_snap.ins.current_select);
        }
    }
}