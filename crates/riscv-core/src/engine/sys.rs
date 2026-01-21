use crate::core::Registers;
use crate::device::bus::{Bus, SystemBus};
use crate::error::RiscVError;

struct SysCallId;

impl SysCallId {
    const WRITE: u32 = 64;
    const EXIT: u32 = 93;
}

pub fn syscall(bus: &mut SystemBus, reg: &Registers, id: u32) -> Result<(), RiscVError> {
    match id {
        SysCallId::WRITE => {
            // System write (print)
            let fd = reg[10];
            let ptr = reg[11];
            let len = reg[12] as usize;

            if fd == 1 {
                // stdout
                let mut slice = vec![0; len];
                bus.read_bytes(ptr, len, &mut slice)?;
                let s = String::from_utf8_lossy(&slice);
                print!("{}", s);
            }

            Ok(())
        },
        SysCallId::EXIT => {
            let exit_code = reg[10];

            Err(RiscVError::SystemExit(exit_code))
        },
        _ => Err(RiscVError::NotImplementedSysCall(id)),
    }
}
