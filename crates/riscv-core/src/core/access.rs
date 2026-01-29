use std::marker::PhantomData;

use crate::exception::Exception;

use AccessType::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessType {
    Load,
    Store,
    Fetch,
    #[cfg(feature = "a")]
    Amo,
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

impl<T> Access<T> {
    pub fn new(addr: u32, kind: AccessType) -> Self {
        Access { addr, kind, _marker: PhantomData }
    }
    
    pub fn into_access_exception(self) -> Exception {
        match self.kind {    
            Load  => Exception::LoadAccessFault(self.addr),
            Store => Exception::StoreOrAmoAccessFault(self.addr),
            Fetch => Exception::InstructionAccessFault(self.addr),
            #[cfg(feature = "a")]
            Amo   => Exception::StoreOrAmoAccessFault(self.addr),
        }
    }

    #[cfg(feature = "s")]
    pub fn into_page_exception(self) -> Exception {
        match self.kind {
            Load => Exception::LoadPageFault(self.addr),
            Store => Exception::StoreOrAmoPageFault(self.addr),
            Fetch => Exception::InstructionPageFault(self.addr),
            #[cfg(feature = "a")]
            Amo   => Exception::StoreOrAmoPageFault(self.addr),
        }
    }  
}

impl Access<Virtual> {
    #[cfg(feature = "s")]
    pub fn into_physical(self, p_addr: u32) -> Access<Physical> {
        Access { addr: p_addr, kind: self.kind, _marker: PhantomData }
    }

    pub fn bypass(self) -> Access<Physical> {
        Access { addr: self.addr, kind: self.kind, _marker: PhantomData }
    }
}
