use std::thread;
use std::sync::mpsc::{Receiver, Sender};

use anyhow::Result;

use riscv_core::{Exception, RiscV, RiscVError};
use riscv_core::debug::{DebugInterface, MachineInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmuCmd {
    Stop,
    Step,
    Run,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MachineMode {
    #[default]
    Stay,
    Running,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MachineData {
    pub info: MachineInfo,
    pub reg: Vec<u32>,
    #[cfg(feature = "zicsr")]
    pub csr: Vec<(String, u32)>,
    pub pc: u32,
}

impl MachineData {
    fn new<D: DebugInterface>(mach: &D) -> Self {
        let reg = mach.inspect_regs().into_iter().collect();
        #[cfg(feature = "zicsr")]
        let csr = mach.inspect_csrs();
        let pc = mach.inspect_pc();
        let info = mach.get_info();

        MachineData { info, reg, csr, pc }
    }

}

#[derive(Debug, Clone, PartialEq)]
pub enum MachineSender {
    Package(MachineData),
    Exception(Exception),
    Bus(Vec<u8>),
    Error(RiscVError)
}

pub fn spawn_machine_thread(mut mach: RiscV, mach_rx: Receiver<EmuCmd>, data_tx: Sender<MachineSender>) {
    thread::spawn(move || -> Result<()> {
        let mut mach_mode = MachineMode::default();

        loop {
            match mach_rx.try_recv() {
                Ok(EmuCmd::Stop) => mach_mode = MachineMode::Stay,
                Ok(EmuCmd::Step) => {
                    match mach.step() {
                        Ok(Some(except)) => {
                            data_tx.send(MachineSender::Exception(except))?;
                        },
                        Err(e) => data_tx.send(MachineSender::Error(e))?,
                        _ => {},
                    }
                    data_tx.send(MachineSender::Package(MachineData::new(&mach)))?;
                }
                Ok(EmuCmd::Run) => mach_mode = MachineMode::Running,
                Ok(EmuCmd::Quit) => return Ok(()),
                Err(_) => {},
            }

            if mach_mode == MachineMode::Running {
                match mach.step() {
                    Ok(Some(except)) => {
                        data_tx.send(MachineSender::Exception(except))?;
                    },
                    Err(e) => data_tx.send(MachineSender::Error(e))?,
                    _ => {},
                }
                data_tx.send(MachineSender::Package(MachineData::new(&mach)))?;
            }

        }
    });
}