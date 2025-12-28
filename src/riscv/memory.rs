use crate::utils::exception::RiscVError;

pub struct Memory {
    size: usize,
    space: Vec<u32>,
}

impl Memory {
    pub fn new(size: usize) -> Self {
        Memory{size, space: vec![0; size]}
    }
    
    pub fn read(&self, idx: usize) -> Result<u32, RiscVError> {
        if idx >= self.size {
            return Err(RiscVError::OutOfBoundMemory);
        }

        Ok(self.space[idx])
    } 

    pub fn write(&mut self, idx: usize, data: u32) -> Result<(), RiscVError> {
        if idx >= self.size {
            return Err(RiscVError::OutOfBoundMemory);
        }

        self.space[idx] = data;
        Ok(())
    } 

}