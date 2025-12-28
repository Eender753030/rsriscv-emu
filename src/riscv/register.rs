use crate::utils::exception::RiscVError;

#[derive(Debug)]
pub struct Registers {
    registers: [u32; 32],
} 

impl Registers {
    pub fn new() -> Self {
        Registers{registers: [0; 32]}
    }

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