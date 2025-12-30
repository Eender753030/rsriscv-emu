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
        if !pc.is_multiple_of(4) {
            return Err(RiscVError::InstructionAddressMisaligned(pc));
        }

        let idx = pc as usize;

        if idx + 4 > self.size {
            return Err(RiscVError::OutOfBoundMemory);
        }

        let slice = &self.space[idx..idx+4];
        Ok(u32::from_le_bytes(slice.try_into().unwrap()))
    } 

    pub fn load(&mut self, start_address:usize, data_container: &[u8]) -> Result<(), RiscVError> {
        let end_address = start_address + data_container.len();

        if end_address > self.size {
            return Err(RiscVError::OutOfBoundMemory);
        }

        self.space[start_address..end_address].copy_from_slice(data_container);
 
        Ok(())
    } 

    pub fn read(&self, address: u32, bytes_amount: usize, is_signed: bool) -> Result<u32, RiscVError> {
        let idx = address as usize;

        if idx + bytes_amount >= self.size  {
            return Err(RiscVError::OutOfBoundMemory);
        }

        let slice = &self.space[idx..idx+bytes_amount];
        let mut four_bytes = [0_u8; 4];

        four_bytes[..bytes_amount].copy_from_slice(slice);

        if is_signed && (slice.last().unwrap() & 0x80 != 0) {
            four_bytes[bytes_amount..].fill(0xff);
        }

        Ok(u32::from_le_bytes(four_bytes))
    }

    pub fn read_batch(&self, address: usize, bytes_amount: usize) -> Result<&[u8], RiscVError> {
        if address + bytes_amount >= self.size  {
            Err(RiscVError::OutOfBoundMemory)
        } else {
            Ok(&self.space[address..address+bytes_amount])
        }
    } 

    pub fn write(&mut self, address: u32, data: u32, bytes_amount: usize) -> Result<(), RiscVError> {
        let idx = address as usize;

        if idx + bytes_amount >= self.size {
            return Err(RiscVError::OutOfBoundMemory);
        }

        let write_data = data.to_le_bytes();
         
        self.space[idx..idx+bytes_amount].copy_from_slice(&write_data[0..bytes_amount]);
        
        Ok(())
    } 
}