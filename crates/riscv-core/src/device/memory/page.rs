pub const PAGE_SIZE: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Page {
    pub space: [u8; PAGE_SIZE],
}

impl Default for Page {
    fn default() -> Self {
        Page { space: [0; PAGE_SIZE] }
    }
}

impl std::ops::Deref for Page {
    type Target = [u8; PAGE_SIZE];
    fn deref(&self) -> &Self::Target {
        &self.space
    }
}

impl std::ops::DerefMut for Page {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.space
    }
}