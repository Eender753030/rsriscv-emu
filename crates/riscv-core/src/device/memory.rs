//! Memory define and implement for Risc-V

use std::ops::{Deref, DerefMut};

use crate::error::RiscVError;

use super::bus::Bus;

const PAGE_SIZE: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Page {
    space: [u8; PAGE_SIZE],
}

impl Default for Page {
    fn default() -> Self {
        Page {space: [0; PAGE_SIZE]}
    }
}

impl Deref for Page {
    type Target = [u8; PAGE_SIZE];

    fn deref(&self) -> &Self::Target {
        &self.space
    }
}

impl DerefMut for Page {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.space
    }
}

/// Memory structure. Store `u8` data as Little Endian.
/// Unit of `size` is byte
#[derive(Debug, Clone, PartialEq)]
pub struct Memory {
    pub size: usize,
    pages: Vec<Option<Box<Page>>>
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
        let aligned_size = size.max(PAGE_SIZE).next_multiple_of(PAGE_SIZE);
        let mut pages = Vec::with_capacity(aligned_size / PAGE_SIZE);
        for _ in 0..aligned_size {
            pages.push(None);
        }
        Memory {
            size: aligned_size,
            pages
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
    pub fn reset(&mut self) {   
        self.pages.fill(None);
    }

    fn translate(&mut self, addr: usize) -> &mut Page {
        let idx = addr / PAGE_SIZE;

        if self.pages[idx].is_none() {
            self.pages[idx] = Some(Box::new(Page::default()));
        }

        // Safe
        self.pages[idx].as_mut().unwrap()
    }
}

const _4GB: usize = 4 * 1024 * 1024 * 1024; 

impl Default for Memory {
    fn default() -> Self {
        Self::new(_4GB)
    }
}

impl Bus for Memory {
    fn read_byte(&mut self, addr: u32) -> Result<u8, RiscVError> {
        let addr = addr as usize;

        if addr >= self.size {
            Err(RiscVError::OutOfBoundMemory)
        } else {
            let page = self.translate(addr);
            Ok(page[addr % PAGE_SIZE])
        }
    }

    fn write_byte(&mut self, addr: u32, data: u8) -> Result<(), RiscVError> {
        let addr = addr as usize;

        if addr >= self.size {
            Err(RiscVError::OutOfBoundMemory)
        } else {
            let page = self.translate(addr);
            page[addr % PAGE_SIZE] = data;
            Ok(())
        }
    }

    fn read_bytes(&mut self, addr: u32, size: usize, des: &mut [u8]) -> Result<(), RiscVError> {
        let addr = addr as usize;

        if addr + size > self.size {
           return Err(RiscVError::OutOfBoundMemory);
        } 
 
        let mut start = 0;
        let mut curr_addr = addr;

        while start < size {
            let p_start = curr_addr % PAGE_SIZE;
            let remain = PAGE_SIZE - p_start;
            let len = std::cmp::min(des.len() - start, remain);

            let page = self.translate(curr_addr);

            des[start..start+len].copy_from_slice(&page[p_start..p_start+len]);

            start += len;
            curr_addr += len; 
        }

        Ok(())
    }

    fn write_bytes(&mut self, addr: u32, size: usize, src: &[u8]) -> Result<(), RiscVError> {
        let addr = addr as usize;

        if addr + size > self.size {
            return Err(RiscVError::OutOfBoundMemory);
        } 

        let mut start = 0;
        let mut curr_addr = addr;

        while start < size {
            let p_start = curr_addr % PAGE_SIZE;
            let remain = PAGE_SIZE - p_start;
            let len = std::cmp::min(src.len() - start, remain);

            let page = self.translate(curr_addr);

            page[p_start..p_start+len].copy_from_slice(&src[start..start+len]);

            start += len;
            curr_addr += len; 
        }

        Ok(())
    }
}

#[cfg(test)]
mod memory_tests {
    use crate::device::memory::_4GB;
    use crate::device::memory::PAGE_SIZE;

    use super::Bus;
    use super::Memory;

    #[test]
    fn create_test() {
        let mem = Memory::new(_4GB);

        assert_eq!(mem.size, _4GB);
        assert_eq!(mem.pages, vec![None; _4GB / PAGE_SIZE]);
    }

    #[test]
    fn read_write_test() {
        let mut mem = Memory::new(PAGE_SIZE * 2);

        // Part 1: test byte
        assert_eq!(mem.write_byte(4095, 255), Ok(()));
        assert_eq!(mem.read_byte(4095), Ok(255));
        assert_eq!(mem.pages[0].as_ref().unwrap()[4095], 255);
        assert_eq!(mem.pages[1], None);
        assert!(mem.pages[0].is_some());


        // Part 2: test bytes
        let data = [0xAA, 0xBB, 0xCC, 0xDD];

        let mut des = [0_u8; 4];

        assert_eq!(mem.write_bytes(4095, 4, &data), Ok(()));
        assert_eq!(mem.read_bytes(4095, 4, &mut des), Ok(()));
        assert_eq!(&mem.pages[0].as_ref().unwrap()[4095], &data[0]);
        assert_eq!(&mem.pages[1].as_ref().unwrap()[0..3], &data[1..4]);

        assert_eq!(&des, &data);

        assert!(mem.pages[0].is_some());
        assert!(mem.pages[1].is_some());

    }
}