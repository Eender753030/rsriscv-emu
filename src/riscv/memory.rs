use crate::utils::exception::RiscVError;

#[derive(Debug)]
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

    pub fn read(&self, address: u32, bytes_amount: usize, is_signed: bool) -> Result<u32, RiscVError> {
        let idx = address as usize;

        if idx + bytes_amount >= self.size  {
            return Err(RiscVError::OutOfBoundMemory);
        }
        Ok(
            if is_signed {
                let mut read_data = self.space[idx + bytes_amount] as i8 as i32;
                for i in 1..=bytes_amount {
                    read_data <<= 8;
                    read_data |= (self.space[idx + bytes_amount - i] as u32) as i32;
                }    
                read_data as u32
            } else {
                let mut read_data = self.space[idx + bytes_amount] as u32;
                for i in 1..=bytes_amount {
                    read_data <<= 8;
                    read_data |= self.space[idx + bytes_amount - i] as u32;
                }  
                read_data  
            }
        )
    } 

    pub fn write(&mut self, address: u32, data: u32, bytes_amount: usize) -> Result<(), RiscVError> {
        let idx = address as usize;

        if idx + bytes_amount >= self.size {
            return Err(RiscVError::OutOfBoundMemory);
        }

        let mut write_data = data;

        self.space[idx] = (write_data & 0xff) as u8;
        for i in 1..=bytes_amount {
            write_data >>= 8;
            self.space[idx + i] = (write_data & 0xff) as u8;
        }
        
        Ok(())
    } 
}