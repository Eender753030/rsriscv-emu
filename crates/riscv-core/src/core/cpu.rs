use riscv_decoder::prelude::*;

use riscv_loader::LoadInfo;

use crate::{Exception, Result, RiscVError, StdResult};
use crate::core::Mmu;
use crate::core::access::{Access, AccessType};
use crate::device::bus::SystemBus;
use crate::device::Device;
use crate::debug::*;
use super::{PC, RegisterFile, CsrFile, PrivilegeMode};

#[derive(Clone, PartialEq, Default)]
pub struct Cpu {
    pub(crate) mode: PrivilegeMode,
    pub(crate) regs: RegisterFile,
    pub(crate) pc: PC,
    pub(crate) csrs: CsrFile,
    pub(crate) mmu: Mmu,
    pub(crate) bus: SystemBus,
}

impl Cpu {
    pub fn load_info(&mut self, info: &LoadInfo) -> StdResult<(), RiscVError> {
        for (code, addr) in info.code.iter() {
            self.load(*addr, code)?
        }
        self.set_pc(info.pc_entry);
        
        if let Some(data_vec) = &info.data {
            for (data, addr) in data_vec.iter() {
                self.load(*addr, data)?
            }
        }
        if let Some((start, size)) = &info.bss {
            self.set_mem_zero(*start, *size)?
        }
        if let Some(other_vec) = &info.other {
            for (data, addr) in other_vec.iter() {
                self.load(*addr, data)?
            }
        }
        Ok(())
    }

    pub fn load(&mut self, addr: u32, data: &[u8]) -> StdResult<(), RiscVError> {
        let access = Access::new(addr, AccessType::Store);
        if self.bus.write_bytes(access, data.len(), data).is_err() {
            Err(RiscVError::LoadFailed)
        } else {
            Ok(())
        }
    }

    pub fn set_pc(&mut self, entry: u32) {
        self.pc.set(entry);
    }

    pub fn set_mem_zero(&mut self, addr: u32, size: usize) -> std::result::Result<(), RiscVError> {
        for i in 0..size {
            let access = Access::new(addr + i as u32, AccessType::Store);
            self.bus.write_byte(access, 0)
                .map_err(|_| RiscVError::BssInitFailed)?
        }
        Ok(())
    }

    pub fn run(&mut self) -> StdResult<(), RiscVError> {
        loop { self.step()?; }
    }
 
    pub fn step(&mut self) -> StdResult<Option<Exception>, RiscVError> {
        Ok(if let Err(execpt) = self.cycle() {        
            self.trap_handle(execpt);
            Some(execpt)
        } else {
            None
        })
    }

    fn cycle(&mut self) -> Result<()> {
        let raw = self.fetch()?;
        
        let ins = self.decode(raw)?;
        
        self.execute(ins)?;
        Ok(())
    }

    fn fetch(&mut self) -> Result<u32> {
        let va_access = Access::new(self.pc.get(), AccessType::Fetch);

        let pa_access = self.mmu.translate(va_access, self.mode, self.csrs.check_satp() , &mut self.bus)?;

        self.csrs.pmp_check(pa_access, 4, self.mode).map_err(|e| match e {
            Exception::InstructionAccessFault(_) => Exception::InstructionAccessFault(va_access.addr),
            _ => e
        })?;

        self.bus.read_u32(pa_access).map_err(|e| match e {
            Exception::InstructionAccessFault(_) => Exception::InstructionAccessFault(va_access.addr),
            _ => e
        })
    }

    fn decode(&self, bytes: u32) -> Result<Instruction> {
        decoder::decode(bytes)
            .map_err(|_| Exception::IllegalInstruction(bytes))
    }

    fn execute(&mut self, ins: Instruction) -> Result<()> {
        match ins {
            Instruction::Base(op, data)  => if self.execute_rv32i(op, data)? {
                    return Ok(());
            },
            Instruction::Privileged(op, data)  => if self.execute_privileged(op, data)? {
                return Ok(())
            },
            #[cfg(feature = "m")]
            Instruction::M(op, data)     => self.execute_m(op, data),
            Instruction::Ziscr(op, data) => self.execute_zicsr(op, data)?,
            Instruction::Zifencei(_, _)  => {},          
        }
        self.pc.step();
        Ok(())
    }

    fn trap_handle(&mut self, except: Exception) {
        let (mode, pc) = self.csrs.trap_entry(self.pc.get(), except, self.mode);
        self.pc.directed_addressing(pc);
        self.mode = mode;
    }

    pub fn reset(&mut self) {
        self.mode = PrivilegeMode::default();
        self.regs.reset();
        self.csrs.reset();
        self.pc.reset();
        self.bus.reset_ram();
    }
}

impl DebugInterface for Cpu {
    fn inspect_regs(&self) -> [u32; 32] {
        self.regs.inspect()
    }

    fn inspect_pc(&self) -> u32 {
        self.pc.get()
    }

    fn inspect_csrs(&self) -> Vec<(String, u32)> {
        self.csrs.inspect()
    }

    fn inspect_mem(&self, addr: u32, len: usize) -> Vec<u8> {
        let mut mem: Vec<u8> = vec![0; len]; 
        // Todo: The execption debuger layout
        let access = Access::new(addr, AccessType::Load);
        let _ = self.bus.read_bytes(access, len, &mut mem);
        mem
    }    

    fn get_info(&self) -> MachineInfo {
        let (dram_size, dram_base, page_size) = self.bus.ram_info();

        MachineInfo::new(dram_size, dram_base, page_size)
    }
}

impl std::fmt::Debug for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Cpu {{")?;
        writeln!(f, " PC: {:#08x}", self.pc.get())?;
        write!(f, " Registers {{")?;
        self.regs.iter().enumerate().try_for_each(|(id, regs)|
            write!(f, " x{}: {}", id, *regs as i32)
        )?;
        writeln!(f, " }}")?;
        write!(f, " {:?}", self.bus)
    }
}

#[cfg(test)]
mod tests;
