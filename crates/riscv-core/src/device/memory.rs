//! Memory define and implement for Risc-V

mod page;

use crate::exception::Exception;
use crate::core::{Access, Physical};

use page::Page;
pub use page::PAGE_SIZE;

use super::Device;

/// Memory structure. Store `u8` data as Little Endian.
/// Unit of `size` is byte
#[derive(Clone, PartialEq, Eq)]
pub struct Memory {
    pub size: usize,
    pages: Vec<Option<Box<Page>>>,
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

        self.pages.get(idx)?.as_ref().map(|page| page.as_ref())
    }

    fn translate_mut(&mut self, addr: usize) -> Option<&mut Page> {
        let idx = addr / PAGE_SIZE;

        Some(self.pages.get_mut(idx)?
            .get_or_insert_with(|| Box::new(Page::default())))
    }
}

const _2GB: usize = 2 * 1024 * 1024 * 1024;

impl Default for Memory {
    fn default() -> Self {
        Self::new(_2GB)
    }
}

impl Device for Memory {
    fn read_byte(&self, access: Access<Physical>) -> Result<u8, Exception> {
        let addr = access.addr as usize;

        if let Some(page) = self.translate(addr) {
            Ok(page[addr % PAGE_SIZE])
        } else {
            Err(access.to_access_exception())
        }
    }

    fn write_byte(&mut self, access: Access<Physical>, data: u8) -> Result<(), Exception> {
        let addr = access.addr as usize;

        if let Some(page) = self.translate_mut(addr) {
            page[addr % PAGE_SIZE] = data;
            Ok(())
        } else {
            Err(access.to_access_exception())
        }   
    }

    fn read_bytes(&self, mut access: Access<Physical>, size: usize, des: &mut [u8]) -> Result<(), Exception> {
        let mut start = 0;

        while start < size {
            let addr = access.addr as usize;
            let p_start = addr % PAGE_SIZE;
            let remain = PAGE_SIZE - p_start;
            let len = std::cmp::min(size - start, remain);

            let page = self.translate(addr);
            match page {
                None => 
                    return Err(access.to_access_exception()),
                Some(p) => 
                    des[start..start + len].copy_from_slice(&p[p_start..p_start + len]),
            }

            start += len;
            access.addr += len as u32;
        }
        Ok(())
    }

    fn write_bytes(&mut self, mut access: Access<Physical>, size: usize, src: &[u8]) -> Result<(), Exception> {
        let mut start = 0;

        while start < size {
            let addr = access.addr as usize;
            let p_start = addr % PAGE_SIZE;
            let remain = PAGE_SIZE - p_start;
            let len = std::cmp::min(size - start, remain);

            let page = self.translate_mut(addr);
            match page {
                None => 
                    return Err(access.to_access_exception()),
                Some(p) => 
                    p[p_start..p_start + len].copy_from_slice(&src[start..start + len]),
            }
            
            start += len;
            access.addr += len as u32;
        }
        Ok(())
    }
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

// #[cfg(test)]
// mod memory_tests {
//     use crate::device::memory::_2GB;
//     use crate::device::memory::PAGE_SIZE;

//     use super::Device;
//     use super::Memory;

//     #[test]
//     fn create_test() {
//         let mem = Memory::new(_2GB);

//         assert_eq!(mem.size, _2GB);
//         assert_eq!(mem.pages, vec![None; _2GB / PAGE_SIZE]);
//     }

//     #[test]
//     fn read_write_test() {
//         let mut mem = Memory::new(PAGE_SIZE * 2);

//         // Part 1: test byte
//         assert_eq!(mem.write_byte(4095, 255), Ok(()));
//         assert_eq!(mem.read_byte(4095), Ok(255));
//         assert_eq!(mem.pages[0].as_ref().unwrap()[4095], 255);
//         assert_eq!(mem.pages[1], None);
//         assert!(mem.pages[0].is_some());

//         // Part 2: test bytes
//         let data = [0xAA, 0xBB, 0xCC, 0xDD];

//         let mut des = [0_u8; 4];

//         assert_eq!(mem.write_bytes(4095, 4, &data), Ok(()));
//         assert_eq!(mem.read_bytes(4095, 4, &mut des), Ok(()));
//         assert_eq!(&mem.pages[0].as_ref().unwrap()[4095], &data[0]);
//         assert_eq!(&mem.pages[1].as_ref().unwrap()[0..3], &data[1..4]);

//         assert_eq!(&des, &data);

//         assert!(mem.pages[0].is_some());
//         assert!(mem.pages[1].is_some());
//     }
// }
