//! Memory define and implement for Risc-V

use std::ops::{Deref, DerefMut};

use crate::exception::Exception;

use super::bus::Bus;

pub const PAGE_SIZE: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Page {
    space: [u8; PAGE_SIZE],
}

impl Default for Page {
    fn default() -> Self {
        Page { space: [0; PAGE_SIZE] }
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
#[derive(Clone, PartialEq)]
pub struct Memory {
    pub size: usize,
    pages: Vec<Option<Box<Page>>>,
}

impl std::fmt::Debug for Memory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Memory {{ size: {}, pages: {} }}", self.size, self.pages.len())?;
    
        self.pages.iter()
            .enumerate()
            .filter_map(|(page_idx, page_opt)| {
                page_opt.as_deref().map(|page| (page_idx, page))
            })
            .flat_map(|(page_idx, page)| {
                page.space.chunks(16)
                    .enumerate()
                    .filter(|(_, line)| line.iter().any(|&b| b != 0))
                    .map(move |(offset, line)| {
                        let addr = page_idx * PAGE_SIZE + (offset * 16);
                        (addr, line)
                    })
            })
            .try_for_each(|(addr, line)| {
                write!(f, " [0x{:08x}] ", addr)?;

                line.chunks(4).try_for_each(|group| {
                    for byte in group {
                        write!(f, "{:02x}", byte)?;
                    }
                    write!(f, " ")
                })?;

                writeln!(f)
            })
    }
}

impl Memory {
    pub fn new(size: usize) -> Self {
        let aligned_size = size.max(PAGE_SIZE).next_multiple_of(PAGE_SIZE);
        let pages = vec![None; aligned_size / PAGE_SIZE];

        Memory { size: aligned_size, pages }
    }

    /// Reset `Memory`'s `space` by fill 0
    pub fn reset(&mut self) {
        self.pages.fill(None);
    }

    fn translate(&self, addr: usize) -> Option<&Page> {
        let idx = addr / PAGE_SIZE;

        self.pages[idx].as_ref().map(|page| page.as_ref())
    }

    fn translate_mut(&mut self, addr: usize) -> &mut Page {
        let idx = addr / PAGE_SIZE;

        if self.pages[idx].is_none() {
            self.pages[idx].replace(Box::new(Page::default()));
        }

        // Safe
        self.pages[idx].as_mut().unwrap()
    }
}

const _2GB: usize = 2 * 1024 * 1024 * 1024;

impl Default for Memory {
    fn default() -> Self {
        Self::new(_2GB)
    }
}

impl Bus for Memory {
    fn read_byte(&self, addr: u32) -> Result<u8, Exception> {
        let addr = addr as usize;

        if let Some(page) = self.translate(addr) {
            Ok(page[addr % PAGE_SIZE])
        } else {
            Err(Exception::LoadAccessFault)
        }
    }

    fn write_byte(&mut self, addr: u32, data: u8) -> Result<(), Exception> {
        let addr = addr as usize;

        let page = self.translate_mut(addr);
        page[addr % PAGE_SIZE] = data;
        Ok(())
    }

    fn read_bytes(&self, addr: u32, size: usize, des: &mut [u8]) -> Result<(), Exception> {
        let addr = addr as usize;

        let mut start = 0;
        let mut curr_addr = addr;

        while start < size {
            let p_start = curr_addr % PAGE_SIZE;
            let remain = PAGE_SIZE - p_start;
            let len = std::cmp::min(des.len() - start, remain);

            let page = self.translate(curr_addr);

            match page {
                None => return Err(Exception::LoadAccessFault),
                Some(p) => des[start..start + len].copy_from_slice(&p[p_start..p_start + len]),
            }

            start += len;
            curr_addr += len;
        }

        Ok(())
    }

    fn write_bytes(&mut self, addr: u32, size: usize, src: &[u8]) -> Result<(), Exception> {
        let addr = addr as usize;

        let mut start = 0;
        let mut curr_addr = addr;

        while start < size {
            let p_start = curr_addr % PAGE_SIZE;
            let remain = PAGE_SIZE - p_start;
            let len = std::cmp::min(src.len() - start, remain);

            let page = self.translate_mut(curr_addr);

            page[p_start..p_start + len].copy_from_slice(&src[start..start + len]);

            start += len;
            curr_addr += len;
        }

        Ok(())
    }
}

#[cfg(test)]
mod memory_tests {
    use crate::device::memory::_2GB;
    use crate::device::memory::PAGE_SIZE;

    use super::Bus;
    use super::Memory;

    #[test]
    fn create_test() {
        let mem = Memory::new(_2GB);

        assert_eq!(mem.size, _2GB);
        assert_eq!(mem.pages, vec![None; _2GB / PAGE_SIZE]);
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
