use crate::utils::exception::RiscVError;
use super::{Reset, Dump};

#[derive(Debug)]
pub struct Registers {
    registers: [u32; 32],
} 

impl Registers {
    pub fn read(&self, id: usize) -> Result<u32, RiscVError> {
        if id == 0 {
            return Ok(0);
        } 

        if id > 31 {
            return Err(RiscVError::InvalidRegister(id));
        } 

        Ok(self.registers[id])
    }

    pub fn write(&mut self, id: usize, data: u32) -> Result<(), RiscVError> {
        if id == 0 {
            return Ok(());
        }

        if id > 31 {
            return Err(RiscVError::InvalidRegister(id));
        } 

        self.registers[id] = data;


        Ok(())
    } 
}

impl Default for Registers {
    fn default() -> Self {
        Registers{registers: [0; 32]}
    }
}

impl Reset for Registers {
    fn reset(&mut self) {
        self.registers.fill(0);
    }
}

impl Dump<i32> for Registers {
    fn dump(&self) -> Vec<i32> {
        self.registers.iter().map(|&x| x as i32).collect()
    }
}