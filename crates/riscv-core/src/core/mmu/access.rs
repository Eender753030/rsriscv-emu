use std::marker::PhantomData;

use crate::exception::Exception;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessType {
    Load,
    Store,
    Fetch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Virtual;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Physical;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Access<T = Virtual> {
    pub addr: u32,
    pub kind : AccessType,  
    _marker: PhantomData<T>,
}

impl <T>Access<T> {
    pub fn new(addr: u32, kind: AccessType) -> Self {
        Access { addr, kind, _marker: PhantomData }
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

impl Access<Virtual> {
    pub fn into_physical(self, p_addr: u32) -> Access<Physical> {
        Access { addr: p_addr, kind: self.kind, _marker: PhantomData }
    }

    pub fn bypass(self) -> Access<Physical> {
        Access { addr: self.addr, kind: self.kind, _marker: PhantomData }
    }
}

