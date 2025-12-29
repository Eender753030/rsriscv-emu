use crate::utils::exception::RiscVError;

pub struct Memory {
    size: usize,
    space: Vec<u8>,
}

impl Memory {
    pub fn new(size: usize) -> Self {
        if size < 4 {
            Memory{size: 4, space: vec![0; 4]}
        } else {
            Memory{size, space: vec![0; size]}
        }
    }
    
    pub fn fetch(&self, pc: u32) -> Result<u32, RiscVError> {
        if pc % 4 != 0 {
            return Err(RiscVError::InstructionAddressMisaligned(pc));
        }

        let idx = pc as usize;

        if idx >= self.size - 3  {
            return Err(RiscVError::OutOfBoundMemory);
        }

        let mut read_data:u32 = self.space[idx + 3] as u32;


        for i in 1..4 {
            read_data <<= 8;
            read_data |= self.space[idx + 3 - i] as u32;
        }    

        Ok(read_data)
    } 

    pub fn load(&mut self, start_address:usize, data_container: &[u8]) -> Result<(), RiscVError> {
        for (i, &data) in data_container.iter().enumerate() {
            let idx = start_address + i;
            if idx > self.size {
                return Err(RiscVError::OutOfBoundMemory);
            }
            
            self.space[start_address + i] = data;
        }
  
        Ok(())
    } 

}