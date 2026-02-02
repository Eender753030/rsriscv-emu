mod debug;

use riscv_decoder::prelude::*;

use riscv_loader::LoadInfo;

use crate::{Exception, Result, RiscVError, StdResult};
#[cfg(feature = "s")]
use crate::core::Mmu;
#[cfg(feature = "zicsr")]
use crate::core::csr::CsrFile;
#[cfg(feature = "zicsr")]
use crate::core::privilege::PrivilegeMode;
use crate::core::access::{Access, AccessType};
use crate::device::bus::SystemBus;
use crate::device::Device;


use super::{PC, RegisterFile};
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Cpu {
    #[cfg(feature = "zicsr")]
    pub(crate) mode: PrivilegeMode,
    pub(crate) regs: RegisterFile,
    pub(crate) pc: PC,
    #[cfg(feature = "zicsr")]
    pub(crate) csrs: CsrFile,
    #[cfg(feature = "s")]
    pub(crate) mmu: Mmu,
    pub(crate) bus: SystemBus,
    #[cfg(feature = "a")]
    pub(crate) reservation: Option<u32>,
    #[cfg(feature = "c")]
    pub(crate) is_compress: bool,
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
        let access = Access::new(addr, AccessType::Store); 
    
        self.bus.write_bytes(access, size, &vec![0; size]).map_err(|_| RiscVError::BssInitFailed)?;
        Ok(())
    }

    pub fn run(&mut self) -> StdResult<(), RiscVError> {
        loop { self.step()?; }
    }
 
    pub fn step(&mut self) -> StdResult<Option<Exception>, RiscVError> {      
        #[cfg(feature = "zicsr")]  
        return Ok(if let Err(execpt) = self.cycle() {      
            self.trap_handle(execpt);
            Some(execpt) 
        } else {
            None
        });
        #[cfg(not(feature = "zicsr"))] 
        Ok(self.cycle().err())
    }

    fn cycle(&mut self) -> Result<()> {
        #[cfg(feature = "c")]
        let ins = if let Some(c_raw) = self.c_fetch()? {
            self.is_compress = true;
            self.decompress(u16::from_le_bytes(c_raw))?
        } else {
            let raw = self.fetch()?;
            self.is_compress = false;
            self.decode(raw)?
        };

        #[cfg(not(feature = "c"))]
        let raw = self.fetch()?;
        #[cfg(not(feature = "c"))]
        let ins = self.decode(raw)?;
        
        self.execute(ins)?;
        Ok(())
    }

    fn fetch(&mut self) -> Result<u32> {
        let va_access = Access::new(self.pc.get(), AccessType::Fetch);

        #[cfg(not(feature = "s"))]
        let pa_access = va_access;

        #[cfg(feature = "s")]
        let pa_access = self.mmu.translate(va_access, self.mode, &self.csrs, &mut self.bus)?;

        #[cfg(feature = "zicsr")]
        self.csrs.pmp_check(pa_access, 4, self.mode).map_err(|e| match e {
            Exception::InstructionAccessFault(_) => Exception::InstructionAccessFault(va_access.addr),
            _ => e
        })?;

        self.bus.read_u32(pa_access).map_err(|e| match e {
            Exception::InstructionAccessFault(_) => Exception::InstructionAccessFault(va_access.addr),
            _ => e
        })
    }

    #[cfg(feature = "c")]
    fn c_fetch(&mut self) -> Result<Option<[u8; 2]>> {
        let va_access = Access::new(self.pc.get(), AccessType::Fetch);

        #[cfg(not(feature = "s"))]
        let pa_access = va_access;

        #[cfg(feature = "s")]
        let pa_access = self.mmu.translate(va_access, self.mode, &self.csrs, &mut self.bus)?;

        #[cfg(feature = "zicsr")]
        self.csrs.pmp_check(pa_access, 2, self.mode).map_err(|e| match e {
            Exception::InstructionAccessFault(_) => Exception::InstructionAccessFault(va_access.addr),
            _ => e
        })?;

        let mut half_raw = [0; 2];
        self.bus.read_bytes(pa_access, 2, &mut half_raw).map_err(|e| match e {
            Exception::InstructionAccessFault(_) => Exception::InstructionAccessFault(va_access.addr),
            _ => e
        })?;

        Ok(if half_raw[0] & 0b11 != 0b11 {
            Some(half_raw)
        } else {
            None
        })
    }

    fn decode(&self, bytes: u32) -> Result<Instruction> {
        decoder::decode(bytes)
            .map_err(|_| Exception::IllegalInstruction(bytes))
    }

    #[cfg(feature = "c")]
    fn decompress(&self, c_bytes: u16) -> Result<Instruction> {
        decoder::decompress(c_bytes)
            .map_err(|_| Exception::IllegalInstruction(c_bytes as u32))
    }

    fn execute(&mut self, ins: Instruction) -> Result<()> {
        match ins {
            Instruction::Base(op, data)  => if self.execute_rv32i(op, data)? {
                    return Ok(());
            },
            #[cfg(feature = "zicsr")]
            Instruction::Privileged(op, data)  => if self.execute_privileged(op, data)? {
                return Ok(())
            },
            #[cfg(feature = "m")]
            Instruction::M(op, data) => self.execute_m(op, data),
            #[cfg(feature = "a")]
            Instruction::A(op, data) => self.execute_a(op, data)?,
            #[cfg(feature = "zicsr")]
            Instruction::Zicsr(op, data, raw) => self.execute_zicsr(op, data, raw)?,
            #[cfg(feature = "zifencei")]
            Instruction::Zifencei(_, _)  => {},          
        }
        #[cfg(feature = "c")]
        if self.is_compress {
            self.pc.half_step();
        } else {
            self.pc.step();
        }
        #[cfg(not(feature = "c"))]
        self.pc.step();
        Ok(())
    }

    #[cfg(feature = "zicsr")]
    fn trap_handle(&mut self, except: Exception) {
        let (mode, pc) = self.csrs.trap_entry(self.pc.get(), except, self.mode);
        self.pc.directed_addressing(pc);
        self.mode = mode;
    }

    pub fn reset(&mut self) {
        self.regs.reset();
        self.pc.reset();
        self.bus.reset_ram();
        #[cfg(feature = "zicsr")] {
            self.mode = PrivilegeMode::default();
            self.csrs.reset();

             #[cfg(feature = "s")]
            self.mmu.reset();
        }

        #[cfg(feature = "a")] {
            self.reservation = None;
        }
        #[cfg(feature = "c")] {
            self.is_compress = false;
        }
    }
}

#[cfg(test)]
mod tests;
