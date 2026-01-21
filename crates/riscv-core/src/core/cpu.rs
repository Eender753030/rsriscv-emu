use riscv_decoder::prelude::*;

use super::{PC, RegisterFile};
use crate::core::csr::CsrFile;
use crate::device::bus::{Bus, SystemBus, DRAM_BASE_ADDR};
use crate::engine::*;
use crate::error::RiscVError;
use crate::exception::Exception;
use crate::debug::*;

#[derive(Clone, PartialEq, Default)]
pub struct Cpu {
    regs: RegisterFile,
    pc: PC,
    csrs: CsrFile,
    bus: SystemBus,
}

impl std::fmt::Debug for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Cpu {{")?;
        writeln!(f, " PC: {:#08x}", self.pc.get())?;
        write!(f, " Registers {{")?;
        self.regs.iter().enumerate().try_for_each(|(id, regs)|
            write!(f, " x{}: {}", id, regs as i32)
        )?;
        writeln!(f, " }}")?;
        write!(f, " {:?}", self.bus)
    }
}

impl Cpu {
    pub fn fisrt_load(&mut self, code: &[u8]) -> Result<(), RiscVError> {
        if let Err(_) = self.bus.write_bytes(DRAM_BASE_ADDR, code.len(), code) {
            Err(RiscVError::LoadCodeFailed)
        } else {
            Ok(())
        }
    }

    pub fn load(&mut self, addr: u32, code: &[u8]) {
        if let Err(e) = self.bus.write_bytes(addr, code.len(), code) {
            self.trap_handle(e)
        }
    }

    pub fn run(&mut self) -> Result<(), RiscVError> {
        loop { self.step()? }
    }
 
    pub fn step(&mut self) -> Result<(), RiscVError> {
        let prev_pc = self.pc.get();
        if let Err(execpt) = self.cycle() {        
            self.trap_handle(execpt);
        }
        if prev_pc == self.pc.get() {
            Err(RiscVError::EndOfInstruction)
        } else {
            Ok(())
        }
    }

    fn cycle(&mut self) -> Result<(), Exception> {
        let raw = self.fetch()?;
        
        let ins = self.decode(raw)?;
        
        self.execute(ins)?;
        Ok(())
    }

    fn fetch(&mut self) -> Result<u32, Exception> {
        self.bus.read_u32(self.pc.get())
    }

    fn decode(&self, bytes: u32) -> Result<Instruction, Exception> {
        decoder::decode(bytes)
            .map_err(|_| Exception::IllegalInstruction)
    }

    fn execute(&mut self, ins: Instruction) -> Result<(), Exception> {
        match ins {
            Instruction::Base(op, data) => {
                if self.execute_rv32i(op, data)? {
                    return Ok(());
                }
            },
            Instruction::Ziscr(op, data) => {
                if self.execute_zicsr(op, data)? {
                    return Ok(());
                }
            },
            Instruction::Zifencei(_, _) => {},
        }
        self.pc.step();
        Ok(())
    }

    fn execute_rv32i(&mut self, op: Rv32iOp, data: InstructionData) -> Result<bool, Exception> {
        let rs1_data = self.regs[data.rs1];
        let rs2_data = self.regs[data.rs2];
        let mut branch = false;
        let mut jump = false;

        match op {
            Rv32iOp::Addi => self.regs.write(data.rd, Alu::add_signed(rs1_data, data.imm)),
            Rv32iOp::Slli => self.regs.write(data.rd, Alu::shl_logic(rs1_data, data.imm as u32)),
            Rv32iOp::Slti => self.regs.write(data.rd, Alu::set_less_than(rs1_data as i32, data.imm)),
            Rv32iOp::Sltiu => self.regs.write(data.rd, Alu::set_less_than_unsigned(rs1_data, data.imm as u32)),
            Rv32iOp::Xori => self.regs.write(data.rd, Alu::xor(rs1_data, data.imm as u32)),
            Rv32iOp::Srli => self.regs.write(data.rd, Alu::shr_logic(rs1_data, data.imm as u32)),
            Rv32iOp::Srai => self.regs.write(data.rd, Alu::shr_ar(rs1_data as i32, data.imm as u32)),
            Rv32iOp::Ori => self.regs.write(data.rd, Alu::or(rs1_data, data.imm as u32)),
            Rv32iOp::Andi => self.regs.write(data.rd, Alu::and(rs1_data, data.imm as u32)),

            Rv32iOp::Add => self.regs.write(data.rd, Alu::add(rs1_data, rs2_data)),
            Rv32iOp::Sub => self.regs.write(data.rd, Alu::sub(rs1_data, rs2_data)),
            Rv32iOp::Sll => self.regs.write(data.rd, Alu::shl_logic(rs1_data, rs2_data)),
            Rv32iOp::Slt => self.regs.write(data.rd, Alu::set_less_than(rs1_data as i32, rs2_data as i32)),
            Rv32iOp::Sltu => self.regs.write(data.rd, Alu::set_less_than_unsigned(rs1_data, rs2_data)),
            Rv32iOp::Xor => self.regs.write(data.rd, Alu::xor(rs1_data, rs2_data)),
            Rv32iOp::Srl => self.regs.write(data.rd, Alu::shr_logic(rs1_data, rs2_data)),
            Rv32iOp::Sra => self.regs.write(data.rd, Alu::shr_ar(rs1_data as i32, rs2_data)),
            Rv32iOp::Or => self.regs.write(data.rd, Alu::or(rs1_data, rs2_data)),
            Rv32iOp::And => self.regs.write(data.rd, Alu::and(rs1_data, rs2_data)),

            Rv32iOp::Lb => self.regs.write(data.rd, Lsu::load(&mut self.bus, rs1_data, data.imm, 1)?),
            Rv32iOp::Lh => self.regs.write(data.rd, Lsu::load(&mut self.bus, rs1_data, data.imm, 2)?),
            Rv32iOp::Lw => self.regs.write(data.rd, Lsu::load(&mut self.bus, rs1_data, data.imm, 4)?),
            Rv32iOp::Lbu => self.regs.write(data.rd, Lsu::load_signed(&mut self.bus, rs1_data, data.imm, 1)?),
            Rv32iOp::Lhu => self.regs.write(data.rd, Lsu::load_signed(&mut self.bus, rs1_data, data.imm, 2)?),

            Rv32iOp::Sb => Lsu::store(&mut self.bus, rs1_data, rs2_data, data.imm, 1)?,
            Rv32iOp::Sh => Lsu::store(&mut self.bus, rs1_data, rs2_data, data.imm, 2)?,
            Rv32iOp::Sw => Lsu::store(&mut self.bus, rs1_data, rs2_data, data.imm, 4)?,

            Rv32iOp::Beq => branch = Branch::equal(rs1_data, rs2_data),
            Rv32iOp::Bne => branch = Branch::not_equal(rs1_data, rs2_data),
            Rv32iOp::Blt => branch = Branch::less(rs1_data as i32, rs2_data as i32),
            Rv32iOp::Bge => branch = Branch::greater_eqaul(rs1_data as i32, rs2_data as i32),
            Rv32iOp::Bltu => branch = Branch::less_unsigned(rs1_data, rs2_data),
            Rv32iOp::Bgeu => branch = Branch::greater_eqaul_unsigned(rs1_data, rs2_data),

            Rv32iOp::Jal => {
                self.regs.write(data.rd, self.pc.get() + 4);
                self.pc.related_addressing(data.imm);
                jump = true;
            },
            Rv32iOp::Jalr => {
                self.regs.write(data.rd, self.pc.get() + 4);
                self.pc.directed_addressing(rs1_data.wrapping_add_signed(data.imm));
                jump = true;
            },

            Rv32iOp::Lui => self.regs.write(data.rd, data.imm as u32),
            Rv32iOp::Auipc => self.regs.write(data.rd, self.pc.get() + data.imm as u32),

            Rv32iOp::Fence => {},

            Rv32iOp::Ecall => {
                return Err(Exception::EnvironmentCallFromMMode);
            },
            Rv32iOp::Ebreak => {},
        }

        if branch {
            self.pc.related_addressing(data.imm);
        }

        Ok(branch | jump)
    }

    fn execute_zicsr(&mut self, op: ZicsrOp, data: InstructionData) -> Result<bool, Exception> {
        let addr = (data.imm & 0xfff) as u16;
        let rs1_data = self.regs[data.rs1];
        let zimm = data.rs1 as u32;

        match op {
            ZicsrOp::Csrrw => {
                if data.rd != 0 {
                    let csr_data = self.csrs.read(addr)?;
                    self.csrs.write(addr, rs1_data)?;
                    self.regs.write(data.rd, csr_data);
                } else {
                    self.csrs.write(addr, rs1_data)?;
                }
            },
            ZicsrOp::Csrrs => {
                let csr_data = self.csrs.read(addr)?;
                if data.rs1 != 0{
                    self.csrs.write(addr, rs1_data | csr_data)?;
                }
                self.regs.write(data.rd, csr_data);
            },
            ZicsrOp::Csrrc => {
                let csr_data = self.csrs.read(addr)?;
                if data.rs1 != 0{
                    self.csrs.write(addr, (!rs1_data) & csr_data)?;
                }
                self.regs.write(data.rd, csr_data);
            },
            ZicsrOp::Csrrwi => {
                if data.rd != 0 {
                    let csr_data = self.csrs.read(addr)?;
                    self.csrs.write(addr, zimm)?;
                    self.regs.write(data.rd, csr_data);
                } else {
                    self.csrs.write(addr, zimm)?;
                } 
            },
            ZicsrOp::Csrrsi => {
                let csr_data = self.csrs.read(addr)?;  
                if zimm != 0{
                    self.csrs.write(addr, zimm | csr_data)?;
                }          
                self.regs.write(data.rd, csr_data);      
            },
            ZicsrOp::Csrrci => {
                let csr_data = self.csrs.read(addr)?;
                if zimm != 0{
                    self.csrs.write(addr, (!zimm) & csr_data)?;
                }        
                self.regs.write(data.rd, csr_data);
            },
            ZicsrOp::Mret => {
                self.pc.directed_addressing(self.csrs.trap_ret());
                return Ok(true);
            }
        }
        
        Ok(false)
    }

    fn trap_handle(&mut self, except: Exception) {
        self.pc.directed_addressing(self.csrs.trap_entry(self.pc.get(), except));
    }

    pub fn reset(&mut self) {
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

    fn inspect_ins(&self, addr: u32, count: usize) -> Vec<(u32, String)> {
        let mut ins_list = Vec::with_capacity(count);
        let mut curr_addr = addr;

        for _ in 0..count {
            let res = self.bus.read_u32(curr_addr)
                .map_err(|_| ())
                .and_then(|raw| 
                    decoder::decode(raw)
                    .map_err(|_| ())
                )
                .map(|ins| ins_list.push((curr_addr, ins.to_string())));

            if let Err(_) = res {
                ins_list.push((curr_addr, "(Unknown)".to_string()));
            }

            curr_addr += 4;
        }

        ins_list
    }

    fn inspect_mem(&self, addr: u32, len: usize) -> Vec<u8> {
        let mut mem: Vec<u8> = vec![0; len]; 
        // Todo: The execption debuger layout
        let _ = self.bus.read_bytes(addr, len, &mut mem);
        mem
    }    
}