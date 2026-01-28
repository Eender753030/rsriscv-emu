use super::TLB_WAY_NUM;
use super::entry::TlbEntry;
use super::plru::PlruState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TlbSet {
    pub entries: [TlbEntry; TLB_WAY_NUM],
    pub plru: PlruState
}

impl TlbSet {
    pub fn get_victim(&mut self) -> (usize, &mut TlbEntry) {
        let mut victim = None;
        for (i, entry) in self.entries.iter().enumerate() {
            if !entry.is_valid() {
                victim = Some(i);
                break;
            }
        }

        let idx = victim.unwrap_or_else(|| self.plru.get_victim());
        
        (idx, &mut self.entries[idx])
    }

    pub fn update(&mut self, way: usize) {
        self.plru.update(way);
    }
}