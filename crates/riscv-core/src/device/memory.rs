//! Memory define and implement for Risc-V

mod page;

use crate::Exception;
use crate::core::{Access, Physical};
use super::Device;

use page::Page;

pub use page::PAGE_SIZE;

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
        self.pages.iter_mut().flatten().for_each(|p| p.fill(0));
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
            Err(access.into_access_exception())
        }
    }

    fn write_byte(&mut self, access: Access<Physical>, data: u8) -> Result<(), Exception> {
        let addr = access.addr as usize;

        if let Some(page) = self.translate_mut(addr) {
            page[addr % PAGE_SIZE] = data;
            Ok(())
        } else {
            Err(access.into_access_exception())
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
                None    => return Err(access.into_access_exception()),
                Some(p) => des[start..start + len].copy_from_slice(&p[p_start..p_start + len]),
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
                    return Err(access.into_access_exception()),
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

#[cfg(test)]
mod tests {
    use crate::Exception;
    use crate::core::{Access, AccessType};
    use crate::device::Device;
    use crate::device::memory::{Memory, _2GB};
    use crate::device::memory::page::PAGE_SIZE;
    
    #[test]
    fn test_initalization() {
        let mem = Memory::new(_2GB);

        assert_eq!(mem.size, _2GB);
        assert_eq!(mem.pages.len(), _2GB / PAGE_SIZE);
        assert!(mem.pages.iter().all(|p| p.is_none()));
    }

    #[test]
    fn test_lazy_alloction() {
        let mut mem = Memory::new(PAGE_SIZE * 2);

        let access = Access::new(0, AccessType::Load);
        assert_eq!(mem.read_byte(access), Err(Exception::LoadAccessFault(0)));

        let access_write = Access::new(0, AccessType::Store);
        assert!(mem.write_byte(access_write, 0xcc).is_ok());

        assert!(mem.pages[0].is_some());
        assert!(mem.pages[1].is_none());

        let access_read = Access::new(0, AccessType::Load);
        assert_eq!(mem.read_byte(access_read), Ok(0xcc));
    }

    #[test]
    fn test_cross_page_access() {
        let mut mem = Memory::new(PAGE_SIZE * 2);
        
        let addr = PAGE_SIZE - 2;
        let data = [0x11, 0x22, 0x33, 0x44];
        
        let access = Access::new(addr as u32, AccessType::Store);
        
        assert!(mem.write_bytes(access, 4, &data).is_ok());
        
        assert!(mem.pages[0].is_some());
        assert!(mem.pages[1].is_some());

        let p0 = mem.pages[0].as_ref().unwrap();
        assert_eq!(p0[PAGE_SIZE - 2], 0x11);
        assert_eq!(p0[PAGE_SIZE - 1], 0x22);

        let p1 = mem.pages[1].as_ref().unwrap();
        assert_eq!(p1[0], 0x33);
        assert_eq!(p1[1], 0x44);
    }
}
