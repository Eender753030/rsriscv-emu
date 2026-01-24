use crate::exception::Exception;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessType {
    Load,
    Store,
    Fetch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Access {
    pub addr: u32,
    pub kind : AccessType,  
}

impl Access {
    pub fn new(addr: u32, kind: AccessType) -> Self {
        Access { addr, kind }
    }
    
    pub fn to_access_exception(self) -> Exception {
        match self.kind {    
            AccessType::Load => Exception::LoadAccessFault(self.addr),
            AccessType::Store => Exception::StoreAccessFault(self.addr),
            AccessType::Fetch => Exception::InstructionAccessFault(self.addr),
        }
    }

    pub fn to_page_exception(self) -> Exception {
        match self.kind {
            AccessType::Load => Exception::LoadPageFault(self.addr),
            AccessType::Store => Exception::StoreOrAmoPageFault(self.addr),
            AccessType::Fetch => Exception::InstructionPageFault(self.addr),
        }
    }
}

