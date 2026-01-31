use riscv_decoder::instruction::AmoInsData;
use riscv_decoder::instruction::AOp::{self, *};

use crate::Result;
use crate::engine::{Alu, Lsu};
use crate::core::cpu::Cpu;

impl Cpu {
    pub(crate) fn execute_a(&mut self, op: AOp, data: AmoInsData) -> Result<()> {
        let rs1_data = self.regs[data.rs1];
        let rs2_data = self.regs[data.rs2];
        
        let rd_data = match op {
            LrW      => self.load_reserved(rs1_data)?,
            ScW      => self.store_conditional(rs1_data, rs2_data)?,
            AmoSwapW => self.atomic_operate(rs1_data, rs2_data, direct_out)?,
            AmoAddW  => self.atomic_operate(rs1_data, rs2_data, Alu::add)?,
            AmoXorW  => self.atomic_operate(rs1_data, rs2_data, Alu::xor)?,
            AmoAndW  => self.atomic_operate(rs1_data, rs2_data, Alu::and)?,
            AmoOrW   => self.atomic_operate(rs1_data, rs2_data, Alu::or)?,
            AmoMinW  => self.atomic_operate(rs1_data, rs2_data, Alu::min)?,
            AmoMaxW  => self.atomic_operate(rs1_data, rs2_data, Alu::max)?,
            AmoMaxuW => self.atomic_operate(rs1_data, rs2_data, Alu::max_unsigned)?,
            AmoMinuW => self.atomic_operate(rs1_data, rs2_data, Alu::min_unsigned)?,    
        };

        self.regs.write(data.rd, rd_data);
        Ok(())
    }   

    fn load_reserved(&mut self, src: u32) -> Result<u32> {
        let mut lsu = Lsu::new(
            &mut self.bus, 
            #[cfg(feature = "s")] &mut self.mmu, 
            #[cfg(feature = "zicsr")] &self.csrs, 
            #[cfg(feature = "zicsr")] self.mode
        );
        let (res_data, addr) = lsu.atomic_load(src)?;
        self.reservation = Some(addr);
        Ok(res_data)
    }

    fn store_conditional(&mut self, des: u32, src: u32) -> Result<u32> {
        let mut lsu = Lsu::new(
            &mut self.bus, 
            #[cfg(feature = "s")] &mut self.mmu, 
            #[cfg(feature = "zicsr")] &self.csrs, 
            #[cfg(feature = "zicsr")] self.mode
        );
        Ok(if lsu.atomic_store(des, src, &mut self.reservation)? {
            0
        } else {
            1
        }) 
    }

    fn atomic_operate<F>(&mut self, des: u32, data: u32, ope: F) -> Result<u32> 
        where F: Fn(u32, u32) -> u32
    {
        let mut lsu = Lsu::new(
            &mut self.bus, 
            #[cfg(feature = "s")] &mut self.mmu, 
            #[cfg(feature = "zicsr")] &self.csrs, 
            #[cfg(feature = "zicsr")] self.mode
        );
        lsu.atomic_operate(des, data, ope, &mut self.reservation)
    }
}

fn direct_out(_: u32, data: u32) -> u32 {
    data
}