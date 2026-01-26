use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LoadInfo {
    pub pc_entry: u32,

    pub code: Vec<(Vec<u8>, u32)>,
    
    pub data: Option<Vec<(Vec<u8>, u32)>>,

    pub bss: Option<(u32, usize)>,

    pub other: Option<Vec<(Vec<u8>, u32)>>,

    pub header_sections: Option<Vec<(String, u32)>>,

    pub symbols: Option<HashMap<u32, String>>,
}

impl LoadInfo {
    pub(crate) fn new(pc_entry: u32, code: Vec<u8>, code_addr: u32) -> Self {
        let code_vec = vec![(code, code_addr)];

        LoadInfo {
            pc_entry, code: code_vec,
            ..Default::default()
        }
    }

    pub(crate) fn from_raw_binary(binary: Vec<u8>) -> Self {
        Self::new(0, binary, 0)
    }

    pub(crate) fn push_code(&mut self, code: Vec<u8>, code_addr: u32) {
        self.code.push((code, code_addr));
    }

    pub(crate) fn push_data(&mut self, data: Vec<u8>, data_addr: u32) {
        self.data
            .get_or_insert_default()
            .push((data, data_addr));
    }

    pub(crate) fn set_bss(&mut self, bss_start: u32, bss_size: usize) {
        self.bss = Some((bss_start, bss_size));
    }

    pub(crate) fn push_other(&mut self, other: Vec<u8>, other_addr: u32) {
        self.other
            .get_or_insert_default()
            .push((other, other_addr));
    }
}