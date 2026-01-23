use std::collections::HashMap;

use riscv_decoder::decoder::decode;
use riscv_loader::LoadInfo;

use crate::instructions::ins_to_string;

pub fn disassembler(info: &LoadInfo) -> Vec<String> {
    let empty_table = HashMap::new();
    let sym_table = info.symbols.as_ref().unwrap_or(&empty_table);

    info.code.iter().flat_map(|(code, base)| 
        code.chunks_exact(4)
            .enumerate()
            .flat_map(move |(i, chunk)| {
                let curr_addr = base + i as u32 * 4;
                let raw = u32::from_le_bytes(chunk.try_into().unwrap());
            
                let mut lines = Vec::new();

                if let Some(sym) = sym_table.get(&curr_addr) {
                    lines.push(format!("{}:", sym));
                }

                let body = decode(raw)
                    .map(|ins| ins_to_string(ins, curr_addr, sym_table))
                    .unwrap_or_else(|_| "(Unknown)".to_string());
                
                lines.push(format!("    {:#010x}: {}", curr_addr, body));
                
                lines
            })
    ).collect()
}
            