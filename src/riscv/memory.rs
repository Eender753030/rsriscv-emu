//! Memory define and implement for Risc-V

use crate::utils::exception::RiscVError;
use super::{Reset, Dump};

/// Memory structure. Store `u8` data as Little Endian.
/// Unit of `size` is byte
#[derive(Debug)]
pub struct Memory {
    size: usize,
    space: Vec<u8>,
}

impl Memory {
    /// Create a `Memory` instance for initialize elements in `space` are 0. 
    /// Force the `size` up to that divides exactly by 4 for align. 
    /// The `size = 1024` means 1KB
    /// ## Example
    /// ```rust,ignore
    /// let mut mem = Memory::new(1024);
    /// // mem's size is 1024
    /// let mut mem = Memory::new(0);
    /// // mem's size is 4 
    /// let mut mem = Memory::new(22);
    /// // mem's size is 24 
    /// ```
    pub fn new(size: usize) -> Self {
        let aligned_size = size.max(4).next_multiple_of(4);
        Memory {
            size: aligned_size, 
            space: vec![0; aligned_size]
        }
    }

    /// Load binary into `Memory`'s `space` start from `start_address`. 
    /// Return `OutOfBoundMemory` if the `end_address` is greater than `Memory`'s `size`.
    /// ## Example
    /// ```rust,ignore
    /// let mut mem = Memory::new(24);
    /// let data: Vec<u8> = vec![1, 2, 3, 4, 5];
    /// 
    /// assert_eq!(mem.load(0, &data), Ok(()));
    /// assert_eq!(mem.load(25, &data), Err(RiscVError::OutOfBoundMemory));
    /// ```
    pub fn load(&mut self, start_address:usize, data_container: &[u8]) -> Result<(), RiscVError> {
        let end_address = start_address + data_container.len();

        if end_address > self.size {
            return Err(RiscVError::OutOfBoundMemory);
        }

        self.space[start_address..end_address].copy_from_slice(data_container);
 
        Ok(())
    } 
    
    /// Fetch `u32` instruction in `Memory`'s `space` start at `pc` pointer.
    /// The `pc` must divides exactly by 4, otherwise return `InstructionAddressMisaligned`
    /// Return `OutOfBoundMemory` if the `end_address` is greater than `Memory`'s `size`.
    /// ## Example
    /// ```rust,ignore
    /// let mut mem = Memory::new(4);
    /// let inst_bytes = vec![0xDD, 0xCC, 0xBB, 0xAA];
    /// mem.load(0, &inst_bytes)?;
    /// 
    /// assert_eq!(mem.fetch(0), Ok(0xAABBCCDD))
    /// assert_eq!(mem.fetch(3), Err(RiscVError::InstructionAddressMisaligned(3)))
    /// assert_eq!(mem.fetch(4), Err(RiscVError::OutOfBoundMemory))
    /// ```
    pub fn fetch(&self, pc: u32) -> Result<u32, RiscVError> {
        if !pc.is_multiple_of(4) {
            return Err(RiscVError::InstructionAddressMisaligned(pc));
        }

        let idx = pc as usize;

        if idx + 4 > self.size {
            return Err(RiscVError::OutOfBoundMemory);
        }

        let slice = &self.space[idx..idx+4];
        // Safe: Here the `slice`'s contents are definitely 4 `u8` by the check on above. 
        // `try_into` and `unwrap` is safe.
        Ok(u32::from_le_bytes(slice.try_into().unwrap()))
    } 

    /// Read a `u32` Little Endian data from `Memory` start at `address` for certain bytes. 
    /// The bytes less than `u32` will auto do sign extension and turn into `u32`.
    /// `bytes_amount` can only is 1 to 4, otherwise return `ReadInvalidBytes`.
    /// Return `OutOfBoundMemory` if the `address + bytes_amount` is greater than or equal to `Memory`'s `size`.
    /// ## Example
    /// ```rust,ignore
    /// let mut mem = Memory::new(8);
    /// let data: Vec<u8> = vec![0xDD, 0xCC, 0xBB, 0xAA];
    /// mem.load(0, &data)?;
    ///
    /// assert_eq!(mem.read(0, 2, false), Ok(0x0000CCDD));
    /// assert_eq!(mem.read(1, 3, true), Ok(0xFFAABBCC));
    /// assert_eq!(mem.read(0, 4, true), Ok(0xAABBCCDD));
    /// 
    /// assert_eq!(mem.read(0, 5, true), Err(RiscVError::ReadInvalidBytes)); 
    /// assert_eq!(mem.read(0, 0, true), Err(RiscVError::ReadInvalidBytes));
    /// assert_eq!(mem.read(2, 3, true), Err(RiscVError::OutOfBoundMemory));  
    /// ```
    pub fn read(&self, address: u32, bytes_amount: usize, is_signed: bool) -> Result<u32, RiscVError> {
        let idx = address as usize;

        if !(1..=4).contains(&bytes_amount) {
            return Err(RiscVError::ReadInvalidBytes);   
        }

        if idx + bytes_amount > self.size  {
            return Err(RiscVError::OutOfBoundMemory);
        }

        let slice = &self.space[idx..idx+bytes_amount];
        let mut four_bytes = [0_u8; 4];

        four_bytes[..bytes_amount].copy_from_slice(slice);

        // Sign extension if MSB is 1.
        // Safe: The `slice` definitely has last element.
        if is_signed && (slice.last().unwrap() & 0x80 != 0) {
            four_bytes[bytes_amount..].fill(0xff);
        }

        Ok(u32::from_le_bytes(four_bytes))
    }

    /// Read a batch(slice) of 'u8' data by size of `bytes_amount` from `Memory` start at `address`.
    /// Return `OutOfBoundMemory` if the `address + bytes_amount` is greater than or equal to `Memory`'s `size`.
    /// ## Example
    /// let mut mem = Memory::new(8);
    /// ```rust,ignore
    /// let data: Vec<u8> = vec![0xDD, 0xCC, 0xBB, 0xAA, 0x99, 0x88, 0x77, 0x66];
    /// mem.load(0, &data)?;
    /// 
    /// assert_eq!(mem.read_batch(0, 8), Ok(&data[0..8])); 
    /// assert_eq!(mem.read_batch(5, 4), Err(RiscVError::OutOfBoundMemory));
    /// ```
    pub fn read_batch(&self, address: usize, bytes_amount: usize) -> Result<&[u8], RiscVError> {
        if address + bytes_amount > self.size  {
            Err(RiscVError::OutOfBoundMemory)
        } else {
            Ok(&self.space[address..address+bytes_amount])
        }
    } 

    /// Write a `u32` `data` into `Memory` start at `address` for certain bytes. 
    /// The input `data`` will turn into `u32` Little Endian.
    /// Return `OutOfBoundMemory` if the `address + bytes_amount` is greater than or equal to `Memory`'s `size`.
    /// ## Example
    /// ```rust,ignore
    /// let mut mem = Memory::new(4);
    /// let data: u32 = 0xAABBCCDD;
    ///
    /// assert_eq!(mem.write(0, data, 3), Ok(()));
    /// assert_eq!(mem.write(0, data, 0), Err(RiscVError::WriteInvalidBytes));
    /// assert_eq!(mem.write(2, data, 3), Err(RiscVError::OutOfBoundMemory));  
    /// ```
    pub fn write(&mut self, address: u32, data: u32, bytes_amount: usize) -> Result<(), RiscVError> {
        let idx = address as usize;

        if bytes_amount == 0 {
            return Err(RiscVError::WriteInvalidBytes);
        }

        if idx + bytes_amount > self.size {
            return Err(RiscVError::OutOfBoundMemory);
        }

        let write_data = data.to_le_bytes();
         
        self.space[idx..idx+bytes_amount].copy_from_slice(&write_data[0..bytes_amount]);
        
        Ok(())
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new(1024)
    }
}

/// Reset `Memory`'s `space` by fill 0
    /// ## Example
    /// # use super::Memory;
    /// ```rust,ignore
    /// let mut mem = Memory::new(4);
    /// let data = vec![0xDD, 0xCC, 0xBB, 0xAA];
    /// mem.load(0, &data).unwrap();
    /// 
    /// mem.reset();
    /// // Now mem'space is vec![0; 4]
    /// ```
impl Reset for Memory {
    fn reset(&mut self) {
        self.space.fill(0);
    }
}

/// Dump `Memory`'s data by a `Vec` of 4 size `u8` arrays.  
/// ## Example
/// ```rust,ignore
/// let mut mem = Memory::new(8);
/// let data1: [u8; 4] = [0xDD, 0xCC, 0xBB, 0xAA];
/// let data2: [u8; 4] = [0x99, 0x88, 0x77, 0x66];
/// mem.load(0, &data1)?;
/// mem.load(4, &data2)?;
///
/// assert_eq!(mem.dump(), vec![data1, data2]);
/// ```
impl Dump<[u8; 4]> for Memory {
    fn dump(&self) -> Vec<[u8; 4]> {
        self.space.chunks(4).map(|slice| slice.try_into().unwrap()).collect()
    }
}

#[cfg(test)]
mod memory_tests {
    use crate::utils::exception::RiscVError;
    use super::{Memory, Reset, Dump};

    #[test]
    fn create_test() {  
        // Test 1: Set size < 4
        let mem = Memory::new(0);
        assert_eq!(mem.size, 4);
        assert_eq!(mem.space.len(), 4);
        assert_eq!(mem.space, vec![0_u8; 4]);

        // Test 2: Set size is not multiple of 4
        let mem = Memory::new(21);
        assert_eq!(mem.size, 24);
        assert_eq!(mem.space.len(), 24);
        assert_eq!(mem.space, vec![0_u8; 24]);

        // Test 3: Set size 1 MB
        let one_mb = 1024 * 1024;
        let mem = Memory::new(one_mb);
        assert_eq!(mem.size, one_mb);
        assert_eq!(mem.space, vec![0_u8; one_mb]);
    }

    #[test]
    fn load_test() {
        // Part 1: Normal function test
        let mut mem = Memory::new(24);
        // Little Endian
        let data:Vec<u8> = vec![0, 1, 2, 3, 4, 5];

        // Test 1: Write at address that in memory's size scope 
        assert_eq!(mem.load(0, &data), Ok(()));
        assert_eq!(&mem.space[0..data.len()], &data);
        // Test 2: Write at memory's start
        assert_eq!(mem.load(10, &data), Ok(()));
        assert_eq!(&mem.space[10..10+data.len()], &data);

        // Test 3: Write at address that will out of bound.
        // length of data is 6. 22 + 6 > 24. Out of bound
        assert_eq!(mem.load(22, &data), Err(RiscVError::OutOfBoundMemory));

        // Part 2: Large data test
        let mut mem_1mb = Memory::new(2_usize.pow(20));

        let large_data:Vec<u8> = (0..=255).collect(); // 256 bytes

        // Test 1: Write at memory's end
        let valid_start = mem_1mb.size - large_data.len();
        assert_eq!(mem_1mb.load(valid_start, &large_data), Ok(()));
        assert_eq!(&mem_1mb.space[valid_start..], &large_data[..]);

        // Test 2: Write at address that will out of bound.
        assert_eq!(mem_1mb.load(valid_start + 1, &large_data), Err(RiscVError::OutOfBoundMemory));
    }

    #[test]
    fn fetch_test() {
        let mut mem = Memory::new(24);
        // Little Endian  0xAABBCCDD -> DD CC BB AA
        let inst_bytes = vec![0xDD, 0xCC, 0xBB, 0xAA];
        mem.load(0, &inst_bytes).unwrap();

        // Test 1: Normal fetch
        assert_eq!(mem.fetch(0), Ok(0xAABBCCDD));
        
        // Test 2: Misaligned fetch
        match mem.fetch(1) {
            Err(RiscVError::InstructionAddressMisaligned(addr)) => assert_eq!(addr, 1),
            _ => panic!("Should return InstructionAddressMisaligned"),
        }

        // Test 3: Out of bound fetch
        assert_eq!(mem.fetch(20), Ok(0)); // Safe (20..24)
        assert_eq!(mem.fetch(24), Err(RiscVError::OutOfBoundMemory)); // Boom (24..28)
    }

    #[test]
    fn read_test() {
        let mut mem = Memory::new(8);
        // Little Endian 0xAABBCCDD 0x66778899 -> [DD CC BB AA] [99 88 77 66]
        let data: Vec<u8> = vec![0xDD, 0xCC, 0xBB, 0xAA, 0x99, 0x88, 0x77, 0x66];
        mem.load(0, &data).unwrap();
        

        // Part 1: read test
        // Test 1: Read unsigned 2 bytes
        assert_eq!(mem.read(0, 2, false), Ok(0x0000CCDD));

        // Test 2: Read signed 3 bytes for sign extension
        assert_eq!(mem.read(1, 3, true), Ok(0xFFAABBCC));

        // Test 3: Read over 4 bytes
        assert_eq!(mem.read(0, 4, true), Ok(0xAABBCCDD)); // Safe
        assert_eq!(mem.read(0, 5, true), Err(RiscVError::ReadInvalidBytes)); // Boom
        assert_eq!(mem.read(0, 0, true), Err(RiscVError::ReadInvalidBytes)); // Boom

        // Test 4: Out of bound fetch
        assert_eq!(mem.read(5, 4, true), Err(RiscVError::OutOfBoundMemory));

        // Part 2: read batch test
        // Test 1: Read 6 bytes
        assert_eq!(mem.read_batch(0, 6), Ok(&data[0..6]));

        // Test 2: Read bytes that over memory size 
        assert_eq!(mem.read_batch(0, 8), Ok(&data[0..8])); // Safe
        assert_eq!(mem.read_batch(5, 4), Err(RiscVError::OutOfBoundMemory)); // Boom
    }

    #[test]
    fn write_test() {
        let mut mem = Memory::new(8);
        let data: u32 = 0xAABBCCDD;

        // Test 1: Normal case
        assert_eq!(mem.write(0, data, 3), Ok(()));
        assert_eq!(mem.space[0..3], [0xDD, 0xCC, 0xBB]);

        assert_eq!(mem.write(4, data, 4), Ok(()));
        assert_eq!(mem.space[4..8], [0xDD, 0xCC, 0xBB, 0xAA]);

        // Test 2: Write 0 byte
        assert_eq!(mem.write(7, data, 0), Err(RiscVError::WriteInvalidBytes));

        // Test 2: Write bytes that over memory size
        assert_eq!(mem.write(6, data, 3), Err(RiscVError::OutOfBoundMemory));  
    }

    #[test]
    fn reset_test() {
        let mut mem = Memory::new(4);
        let data: Vec<u8> = vec![0xDD, 0xCC, 0xBB, 0xAA];
        mem.load(0, &data).unwrap();

        assert_eq!(&mem.space, &data);

        // Test: Reset memory
        mem.reset();
        assert_eq!(mem.space, vec![0; 4]);
    }

    #[test]
    fn dump_test() {     
        let mut mem = Memory::new(8);
        let data1: [u8; 4] = [0xDD, 0xCC, 0xBB, 0xAA];
        let data2: [u8; 4] = [0x99, 0x88, 0x77, 0x66];
        mem.load(0, &data1).unwrap();
        mem.load(4, &data2).unwrap();
        
        // Test: Dump memory
        assert_eq!(mem.dump(), vec![data1, data2]);
    }
}