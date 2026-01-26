use std::collections::HashMap;

use riscv_decoder::instruction::Instruction::{self, *};

use crate::csr_addr::CsrAddr;

pub fn ins_to_string(ins: Instruction, addr: u32, sym_table: &HashMap<u32, String>) -> String {   
    match ins {
        Base(op, data) => {
            if op.is_itype_ar() {
                format!("{:<7} x{}, x{}, {}", op, data.rd, data.rs1, data.imm)
            } else if op.is_itype_load() | op.is_itype_jump() {
                format!("{:<7} x{}, {}(x{})", op, data.rd, 
                    sym_table.get(&(addr.wrapping_add_signed(data.imm))).unwrap_or(&data.imm.to_string()), 
                    data.rs1)
            } else if op.is_itype_fence() {
                match data.imm & 0xf00 {
                    0b0000 => {
                        let succ = check_fence(data.imm);
                        let pred = check_fence(data.imm >> 4);
                        if succ == "iorw" && pred == "iorw" {
                            format!("{:<7}", op)
                        } else {
                            format!("{:<7} {}, {}", op, pred, succ)
                        }
                    },
                    0b1000 => format!("{:<7}", "fence.tso"),
                    _ => format!("{:<7} (Unknown format)", op)
                }               
            } else if op.is_itype_system() {
                format!("{:<7}", op)
            } else if op.is_stype() {
                format!("{:<7} x{}, {}(x{})", op, data.rs2, data.imm, data.rs1)
            } else if op.is_btype() {
                format!("{:<7} x{}, x{}, {}", op, data.rs1, data.rs2, data.imm)
            } else if op.is_jtype() {
                format!("{:<7} x{}, {}", op, data.rd, 
                    sym_table.get(&(addr.wrapping_add_signed(data.imm))).unwrap_or(&data.imm.to_string())
                )
            } else if op.is_utype() {
                format!("{:<7} x{}, {:#x}", op, data.rd, (data.imm as u32) >> 12)
            } else { // rtype
                format!("{:<7} x{}, x{}, x{}", op, data.rd, data.rs1, data.rs2)
            }
        },
        Privileged(op) => {
            format!("{:<7}", op)
        }
        M(op, data) => {
            format!("{:<7} x{}, x{}, x{}", op, data.rd, data.rs1, data.rs2)
        },
        Ziscr(op, data) => {
            let csr_str = CsrAddr::try_from(data.imm as u32 & 0xfff)
                .map(|addr| addr.to_string())
                .unwrap_or_else(|addr| format!("{:#x}",addr));

            format!("{:<7} x{}, {}, x{}", op, data.rd, csr_str, data.rs1)       
        },
        Zifencei(op, _)=> {
            format!("{:<7}", op)
        },
    }
}

fn check_fence(data: i32) -> String {
    let mut output = String::new();
    let mut set = "iorw".chars();

    let mut mask = 0x8;
    
    for _ in 0..4 {
        let mode = set.next().unwrap();
        if (data & mask) != 0 {
            output.push(mode);
        }
        mask >>= 1;
    }

    output
}

#[cfg(test)]
mod tests {
    use crate::instructions::*;
    use riscv_decoder::instruction::{Instruction, InstructionData, Rv32iOp};
    use std::collections::HashMap;

    #[test]
    fn test_base_instruction_disasm() {
        let sym_table = HashMap::new();
        let addr = 0x80000000;

        // addi x10, x0, 10
        let ins = Instruction::Base(
            Rv32iOp::Addi,
            InstructionData { rd: 10, rs1: 0, rs2: 0, imm: 10 }
        );
        assert_eq!(ins_to_string(ins, addr, &sym_table), "addi    x10, x0, 10");

        // sw x5, 4(x6)
        let ins = Instruction::Base(
            Rv32iOp::Sw,
            InstructionData { rd: 0, rs1: 6, rs2: 5, imm: 4 }
        );
        assert_eq!(ins_to_string(ins, addr, &sym_table), "sw      x5, 4(x6)");
    }

    #[test]
    fn test_disasm_with_symbols() {
        let mut sym_table = HashMap::new();
        let addr = 0x80000000;
        let target_addr = 0x80000100;
        sym_table.insert(target_addr, "target_label".to_string());

        // jal x1, 256
        let ins = Instruction::Base(
            Rv32iOp::Jal,
            InstructionData { rd: 1, rs1: 0, rs2: 0, imm: 256 }
        );
        
        let result = ins_to_string(ins, addr, &sym_table);
        assert_eq!(result, "jal     x1, target_label");
    }
}
