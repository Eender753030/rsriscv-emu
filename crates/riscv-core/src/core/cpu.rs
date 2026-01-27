use riscv_decoder::prelude::*;

use riscv_loader::LoadInfo;

use super::{PC, RegisterFile, CsrFile, PrivilegeMode};
use crate::core::{Access, AccessType, Mmu};
use crate::device::bus::SystemBus;
use crate::device::Device;
use crate::error::RiscVError;
use crate::exception::Exception;
use crate::debug::*;

#[derive(Clone, PartialEq, Default)]
pub struct Cpu {
    pub(crate) mode: PrivilegeMode,
    pub(crate) regs: RegisterFile,
    pub(crate) pc: PC,
    pub(crate) csrs: CsrFile,
    pub(crate) bus: SystemBus,
}

impl Cpu {
    pub fn load_info(&mut self, info: &LoadInfo) -> Result<(), RiscVError> {
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

    pub fn load(&mut self, addr: u32, data: &[u8]) -> Result<(), RiscVError> {
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

    pub fn set_mem_zero(&mut self, addr: u32, size: usize) -> Result<(), RiscVError> {
        for i in 0..size {
            let access = Access::new(addr + i as u32, AccessType::Store);
            self.bus.write_byte(access, 0)
                .map_err(|_| RiscVError::BssInitFailed)?
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), RiscVError> {
        loop { self.step()?; }
    }
 
    pub fn step(&mut self) -> Result<Option<Exception>, RiscVError> {
        Ok(if let Err(execpt) = self.cycle() {        
            self.trap_handle(execpt);
            Some(execpt)
        } else {
            None
        })
    }

    fn cycle(&mut self) -> Result<(), Exception> {
        let raw = self.fetch()?;
        
        let ins = self.decode(raw)?;
        
        self.execute(ins)?;
        Ok(())
    }

    fn fetch(&mut self) -> Result<u32, Exception> {
        let va_access = Access::new(self.pc.get(), super::AccessType::Fetch);

        let pa_access = Mmu::translate(va_access, self.mode, self.csrs.check_satp(), &mut self.bus)?;

        self.csrs.pmp_check(pa_access, 4, self.mode).or_else(|e| Err(match e {
            Exception::InstructionAccessFault(_) => Exception::InstructionAccessFault(va_access.addr),
            _ => e
        }))?;

        self.bus.read_u32(pa_access).or_else(|e| Err(match e {
            Exception::InstructionAccessFault(_) => Exception::InstructionAccessFault(va_access.addr),
            _ => e
        }))
    }

    fn decode(&self, bytes: u32) -> Result<Instruction, Exception> {
        decoder::decode(bytes)
            .map_err(|_| Exception::IllegalInstruction(bytes))
    }

    fn execute(&mut self, ins: Instruction) -> Result<(), Exception> {
        match ins {
            Instruction::Base(op, data)  => if self.execute_rv32i(op, data)? {
                    return Ok(());
            },
            Instruction::Privileged(op)  => {
                self.execute_privileged(op);
                return Ok(())
            },
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
mod tests {
    use super::*;
    use crate::constance::DRAM_BASE_ADDR;
    use crate::core::privilege::PrivilegeMode;

    // Helper: 建立一個乾淨的 CPU
    fn new_cpu() -> Cpu {
        Cpu::default()
    }

    #[test]
    fn test_cpu_initial_state() {
        let cpu = new_cpu();
        assert_eq!(cpu.pc.get(), DRAM_BASE_ADDR, "PC should start at DRAM base");
        assert_eq!(cpu.mode, PrivilegeMode::Machine, "Should start in Machine Mode");
        assert_eq!(cpu.regs[1], 0);
    }

    #[test]
    fn test_load_program_to_memory() {
        let mut cpu = new_cpu();
        let code = vec![0xEF, 0xBE, 0xAD, 0xDE]; 
        
        cpu.load(DRAM_BASE_ADDR, &code).expect("Load failed");

        let access = Access::new(DRAM_BASE_ADDR, AccessType::Load);
        let val = cpu.bus.read_u32(access.into_physical(DRAM_BASE_ADDR)).expect("Bus read failed");
        
        assert_eq!(val, 0xDEADBEEF, "Memory content mismatch");
    }

    #[test]
    fn test_cycle_execution_addi() {
        // Fetch-Decode-Execute
        let mut cpu = new_cpu();

        // addi x1, x0, 10
        let code = 0x00A00093u32.to_le_bytes();
        cpu.load(DRAM_BASE_ADDR, &code).unwrap();

        cpu.step().expect("Step failed");

        assert_eq!(cpu.pc.get(), DRAM_BASE_ADDR + 4, "PC did not advance");
        assert_eq!(cpu.regs[1], 10, "x1 register value incorrect");
    }

    #[test]
    fn test_cycle_execution_add() {
        let mut cpu = new_cpu();

        cpu.regs.write(1, 10); // x1 = 10
        cpu.regs.write(2, 20); // x2 = 20

        // add x3, x1, x2
        let code = 0x002081B3u32.to_le_bytes();
        cpu.load(DRAM_BASE_ADDR, &code).unwrap();

        cpu.step().unwrap();

        // x3 = 10 + 20 = 30
        assert_eq!(cpu.regs[3], 30);
    }

    #[test]
    fn test_cycle_execution_bne_taken() {
        let mut cpu = new_cpu();

        cpu.regs.write(1, 5);
        cpu.regs.write(2, 10);

        // bne x1, x2, 8
        let bne_code = 0x00209463u32.to_le_bytes();
        
        cpu.load(DRAM_BASE_ADDR, &bne_code).unwrap();

        cpu.step().unwrap();

        assert_eq!(cpu.pc.get(), DRAM_BASE_ADDR + 8, "Branch did not take");
    }

    #[test]
    fn test_exception_trap_handling() {
        let mut cpu = new_cpu();

        // mtvec = 0x8000_0100
        let handler_base = DRAM_BASE_ADDR + 0x100;
        cpu.csrs.write(0x305, handler_base, PrivilegeMode::Machine).unwrap();

        // Illegal: 0xFFFFFFFF
        let illegal_inst = 0xFFFFFFFFu32.to_le_bytes();
        cpu.load(DRAM_BASE_ADDR, &illegal_inst).unwrap();

        cpu.step().unwrap(); 

        assert_eq!(cpu.pc.get(), handler_base, "Did not trap to mtvec");
  
        let mcause = cpu.csrs.read(0x342, PrivilegeMode::Machine).unwrap();
        assert_eq!(mcause, 2, "mcause wrong");

        let mepc = cpu.csrs.read(0x341, PrivilegeMode::Machine).unwrap();
        assert_eq!(mepc, DRAM_BASE_ADDR, "mepc wrong");
    }
}