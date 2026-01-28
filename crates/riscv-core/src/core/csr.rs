mod addr;
mod mstatus;
mod pmpcfg;
#[cfg(feature = "s")]
mod satp;

use crate::{Exception, Result};
use crate::core::access::{Access, Physical};
use crate::core::privilege::PrivilegeMode;

use addr::CsrAddr;
use mstatus::Mstatus;
use pmpcfg::Pmpcfg;
#[cfg(feature = "s")]
use satp::Satp;

pub(super) const PMPCFG_NUM: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CsrFile {
    #[cfg(feature = "s")] stvec: u32,
    #[cfg(feature = "s")] sepc: u32,
    #[cfg(feature = "s")] scause: u32,
    #[cfg(feature = "s")] sscratch: u32,
    #[cfg(feature = "s")] stval: u32,
    #[cfg(feature = "s")] satp: Satp,

    mstatus: Mstatus,
    #[cfg(feature = "s")] medeleg: u32,
    #[cfg(feature = "s")] mideleg: u32,
    mie: u32,
    mtvec: u32,
    mscratch: u32,
    mepc: u32,
    mcause: u32,
    mtval: u32,
    mip: u32,

    pmpcfg: [Pmpcfg; PMPCFG_NUM],
    pmpaddr: [u32; PMPCFG_NUM * 4],
}

const MODE_MASK: u16 = 3 << 8;
// const INTERRUPT_MASK: u32 = 1 << 31;

impl CsrFile {
    pub fn read(&mut self, addr: u16, mode: PrivilegeMode) -> Result<u32> {    
        if (mode as u16) < ((addr & MODE_MASK) >> 8) {
            Err(Exception::IllegalInstruction(addr as u32))
        } else {
            Ok(match CsrAddr::try_from(addr)? {
                CsrAddr::Ustatus => 0,

                #[cfg(feature = "s")]CsrAddr::Sstatus => self.mstatus.read_s(),
                #[cfg(feature = "s")]CsrAddr::Sie => self.mie & self.mideleg,
                #[cfg(feature = "s")] CsrAddr::Stvec => self.stvec,
                #[cfg(feature = "s")] CsrAddr::Sscratch => self.sscratch,
                #[cfg(feature = "s")] CsrAddr::Sepc => self.sepc,
                #[cfg(feature = "s")] CsrAddr::Scause => self.scause,
                #[cfg(feature = "s")] CsrAddr::Stval => self.stval,
                #[cfg(feature = "s")] CsrAddr::Sip => self.mip & self.mideleg,
                #[cfg(feature = "s")] CsrAddr::Satp => self.satp.into(),

                CsrAddr::Mstatus => self.mstatus.read_m(),
                #[cfg(feature = "s")] CsrAddr::Medeleg => self.medeleg,
                #[cfg(feature = "s")] CsrAddr::Mideleg => self.mideleg,
                CsrAddr::Mie => self.mie,
                CsrAddr::Mtvec => self.mtvec,
                CsrAddr::Mscratch => self.mscratch,
                CsrAddr::Mepc => self.mepc,
                CsrAddr::Mcause => self.mcause,
                CsrAddr::Mtval => self.mtval,
                CsrAddr::Mip => self.mip,

                CsrAddr::Pmpcfg(num) => self.pmpcfg[num].into(),
                CsrAddr::Pmpaddr(num) => self.pmpaddr[num],
                CsrAddr::Mhartid => 0,
            })
        }
    }

    pub fn write(&mut self, addr: u16, data: u32, mode: PrivilegeMode) -> Result<()> {
        if (mode as u16) < ((addr & MODE_MASK) >> 8) {
            Err(Exception::IllegalInstruction(addr as u32))
        } else {
            match CsrAddr::try_from(addr)? {
                CsrAddr::Ustatus => {},

                #[cfg(feature = "s")] CsrAddr::Sstatus => self.mstatus.write_s(data),
                #[cfg(feature = "s")] CsrAddr::Sie => self.mie = (self.mie & !self.mideleg) | (data & self.mideleg),
                #[cfg(feature = "s")] CsrAddr::Stvec => self.stvec = data,
                #[cfg(feature = "s")] CsrAddr::Sscratch => self.sscratch = data,
                #[cfg(feature = "s")] CsrAddr::Sepc => self.sepc = data,
                #[cfg(feature = "s")] CsrAddr::Scause => self.scause = data,
                #[cfg(feature = "s")] CsrAddr::Stval => self.stval = data,
                #[cfg(feature = "s")] CsrAddr::Sip => self.mip = (self.mip & !self.mideleg) | (data & self.mideleg),
                #[cfg(feature = "s")] CsrAddr::Satp => self.satp = data.into(),

                CsrAddr::Mstatus => self.mstatus.write_m(data),
                #[cfg(feature = "s")] CsrAddr::Medeleg => self.medeleg = data,
                #[cfg(feature = "s")] CsrAddr::Mideleg => self.mideleg = data,
                CsrAddr::Mie => self.mie = data,
                CsrAddr::Mtvec => self.mtvec = data,
                CsrAddr::Mscratch => self.mscratch = data,
                CsrAddr::Mepc => self.mepc = data,
                CsrAddr::Mcause => self.mcause = data,
                CsrAddr::Mtval => self.mtval = data,
                CsrAddr::Mip => self.mip = data,

                CsrAddr::Pmpcfg(num) => self.pmpcfg[num] = data.into(),
                CsrAddr::Pmpaddr(num) => self.pmpaddr[num] = data, 
                CsrAddr::Mhartid => return Err(Exception::IllegalInstruction(addr as u32)),
            };
            Ok(())
        }
    }

    pub fn trap_entry(&mut self, curr_pc: u32, except_code: Exception, mode: PrivilegeMode) -> (PrivilegeMode, u32) {
        let target_mode = match mode {
            PrivilegeMode::Machine => PrivilegeMode::Machine,
            #[cfg(feature = "s")]
            PrivilegeMode::Supervisor | PrivilegeMode::User => {
                if self.medeleg & (1 << (u32::from(except_code))) > 0 {
                    PrivilegeMode::Supervisor
                } else {
                    PrivilegeMode::Machine
                }
            }
            #[cfg(not(feature = "s"))]
            PrivilegeMode::User => PrivilegeMode::Machine
        };

        let tval = match except_code {
            Exception::IllegalInstruction(raw) => raw,
            Exception::LoadAccessFault(addr) |
            Exception::StoreAccessFault(addr) |
            Exception::InstructionAccessFault(addr) | 
            Exception::LoadPageFault(addr) |
            Exception::StoreOrAmoPageFault(addr) |
            Exception::InstructionPageFault(addr)
                => addr,
            _   => 0,
        };
   
        match target_mode {
            PrivilegeMode::Machine => {
                self.mepc = curr_pc;
                self.mcause = except_code.into();
                self.mtval = tval;
                self.mstatus.set_mpie(self.mstatus.mie());
                self.mstatus.set_mie(0);
                self.mstatus.set_mpp(mode as u8);
                let base_addr = self.mtvec & !0b11;
                (target_mode,
                if self.mtvec & 0b11 == 0b01 {
                    base_addr + 4 * u32::from(except_code)
                } else {
                    base_addr
                })
            },
            #[cfg(feature = "s")]
            PrivilegeMode::Supervisor => {
                self.sepc = curr_pc;
                self.scause = except_code.into();
                self.stval = tval;
                self.mstatus.set_spie(self.mstatus.sie());
                self.mstatus.set_sie(0);
                self.mstatus.set_spp(mode as u8);
                let base_addr = self.stvec & !0b11;
                (target_mode,
                if self.stvec & 0b11 == 0b01 {
                    base_addr + 4 * u32::from(except_code)
                } else {
                    base_addr
                })
            },
            PrivilegeMode::User => {(PrivilegeMode::User, 0)},
        }
    } 

    pub fn trap_mret(&mut self) -> (PrivilegeMode, u32) {
        let mode = self.mstatus.mpp().into();
        
        self.mstatus.set_mie(self.mstatus.mpie());
        self.mstatus.set_mpie(1);
        self.mstatus.set_mpp(0);

        (mode, self.mepc)
    } 

    #[cfg(feature = "s")]
    pub fn trap_sret(&mut self) -> (PrivilegeMode, u32) {
        let mode = match self.mstatus.spp() {
            0b0 => PrivilegeMode::User,
            _ => PrivilegeMode::Supervisor,
        };
        
        self.mstatus.set_sie(self.mstatus.spie());
        self.mstatus.set_spie(1);
        self.mstatus.set_spp(0);

        (mode, self.sepc)
    } 

    #[cfg(feature = "s")]
    pub fn check_satp(&self) -> Option<(u16, u32)> {
        if self.satp.mode() > 0  {
            Some((self.satp.asid(), self.satp.ppn()))
        } else {
            None
        }
    }

    pub fn pmp_check(&self, access: Access<Physical>, size: usize, mode: PrivilegeMode) -> Result<()> {
        use pmpcfg::MatchingMode::*;
        let mut is_match = None;
        for i in 0..PMPCFG_NUM * 4 {
            if match self.pmpcfg[i / 4][i % 4].mode() {
                Off   => continue,
                Tor   => self.top_of_range(i, access.addr, size),
                Na4   => self.na4(i, access.addr, size),
                Napot => self.napot(i, access.addr, size),
            } {
                is_match = Some(i);
                break;
            }
        }
        match is_match {
            Some(idx) => {
                let pmpcfg = &self.pmpcfg[idx / 4][idx % 4];
                if pmpcfg.mode_check(mode) {
                    return Ok(());
                }
                if !pmpcfg.access_check(access) {
                    return Err(access.into_access_exception());
                }
                Ok(())
            },
            None => match mode {
                PrivilegeMode::Machine => Ok(()),
                _                      => Err(access.into_access_exception()),
            }
        }
    }

    fn top_of_range(&self, idx: usize, addr: u32, size: usize) -> bool {
        let lower = match idx.checked_sub(1) {
            Some(i) => self.pmpaddr[i] << 2,
            None      => 0,
        };
        let upper = self.pmpaddr[idx] << 2;

        lower <= addr && addr + (size as u32) < upper
    }

    fn na4(&self, idx: usize, addr: u32, size: usize) -> bool {
        let base = self.pmpaddr[idx] << 2;

        base <= addr && addr + (size as u32) < base + 4
    }

    fn napot(&self, idx: usize, addr: u32, size: usize) -> bool {
        let mask_bit = self.pmpaddr[idx].trailing_ones();
        let chunck_size = 1 << (3 + mask_bit);
        let base = (self.pmpaddr[idx] & !((1 << mask_bit) - 1)) << 2;

        base <= addr && (addr as usize) + size < (base as usize)  + chunck_size
    }

    pub fn reset(&mut self) {
        *self = Self::default()
    }

    pub fn inspect(&self) -> Vec<(String, u32)> {
        let pmp_list = self.pmpcfg.iter().enumerate()
            .map(|(i, cfg)| (format!("pmpcfg{}", i), (*cfg).into()))
            .chain(self.pmpaddr.iter().enumerate()
                .map(|(i, addr)| (format!("pmpaddr{}", i), *addr))
            );

        let mut csr_list: Vec<(String, u32)> = vec![
            ("ustatus".to_string(), 0),
            ("sstatus".to_string(), self.mstatus.read_s()),
            #[cfg(feature = "s")] ("sie".to_string(), self.mie & self.mideleg),
            #[cfg(feature = "s")] ("stvec".to_string(), self.stvec),
            #[cfg(feature = "s")] ("sscratch".to_string(), self.sscratch),
            #[cfg(feature = "s")] ("sepc".to_string(), self.sepc),
            #[cfg(feature = "s")] ("scause".to_string(), self.scause),
            #[cfg(feature = "s")] ("stval".to_string(), self.stval),
            #[cfg(feature = "s")] ("sip".to_string(), self.mip & self.mideleg),
            #[cfg(feature = "s")] ("stap".to_string(), self.satp.into()),
            ("mstatus".to_string(), self.mstatus.read_m()),
            #[cfg(feature = "s")] ("medeleg".to_string(), self.medeleg),
            #[cfg(feature = "s")] ("mideleg".to_string(), self.mideleg),
            ("mie".to_string(), self.mie),
            ("mtvec".to_string(), self.mtvec),
            ("mscratch".to_string(), self.mscratch),
            ("mepc".to_string(), self.mepc),
            ("mcause".to_string(), self.mcause),
            ("mip".to_string(), self.mip),
        ];
        csr_list.extend(pmp_list);
        csr_list.push(("mhartid".to_string(), 0));

        csr_list
    }
}

#[cfg(test)]
mod tests;