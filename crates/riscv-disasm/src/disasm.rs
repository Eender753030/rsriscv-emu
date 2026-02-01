use std::collections::HashMap;

use riscv_loader::LoadInfo;
use riscv_decoder::decoder::decode;
#[cfg(feature = "c")]
use riscv_decoder::decoder::decompress;

use crate::instructions::ins_to_string;

#[cfg(not(feature = "c"))]
pub fn disassembler(info: &LoadInfo) -> Vec<(u32, String)> {
    let empty_table = HashMap::new();
    let sym_table = info.symbols.as_ref().unwrap_or(&empty_table);

    info.code.iter().flat_map(|(code, base)| 
        code.chunks_exact(4)
            .enumerate()
            .flat_map(move |(i, chunk)| {
                let curr_addr = base + i as u32 * 4;
                let raw = u32::from_le_bytes(chunk.try_into().unwrap());
            
                let mut lines = Vec::new();

                let label = if let Some(sym) = sym_table.get(&curr_addr) {
                    format!("<{}>:\n ", sym)
                } else {
                    String::new()
                };

                let body = decode(raw)
                    .map(|ins| ins_to_string(ins, curr_addr, sym_table))
                    .unwrap_or_else(|_| format!("(Unknown) {:010x}", raw));
                
                lines.push((curr_addr, format!("{}{:#010x}: {}",label, curr_addr, body)));
                
                lines
            })
    ).collect()
}

#[cfg(feature = "c")]
pub fn disassembler(info: &LoadInfo) -> Vec<(u32, String)> {
    let empty_table = HashMap::new();
    let sym_table = info.symbols.as_ref().unwrap_or(&empty_table);

    let mut res = Vec::new();

    for (code_bytes, base_addr) in &info.code {
        let mut offset = 0;
        let len = code_bytes.len();

        while offset < len {
            let curr_addr = base_addr + offset as u32;

            if offset + 2 > len {
                res.push((curr_addr, format!("{:#010x}: (Incomplete)", curr_addr)));
                break;
            }

            let c_raw = u16::from_le_bytes(
                [code_bytes[offset], code_bytes[offset + 1]]
            );

            let is_compress = c_raw & 0b11 != 0b11;

            let (ins_len, body) = if is_compress {
                let body = decompress(c_raw)
                    .map(|ins| format!("(C) {}", ins_to_string(ins, curr_addr, sym_table)))
                    .unwrap_or_else(|_| format!("(Unknown Compress) {:#06x}", c_raw));
                (2, body)
            } else {
                if offset + 4 > len {
                    res.push((curr_addr, format!("{:#010x}: (Incomplete)", curr_addr)));
                    break;
                }

                let raw = u32::from_le_bytes(
                    code_bytes[offset..offset+4].try_into().unwrap()
                );

                let body = decode(raw)
                    .map(|ins| ins_to_string(ins, curr_addr, sym_table))
                    .unwrap_or_else(|_| format!("(Unknown) {:010x}", raw));
                (4, body)
            };

            let mut lines = Vec::new();

            let label = if let Some(sym) = sym_table.get(&curr_addr) {
                format!("{:#010x} <{}>:\n ",curr_addr, sym)
            } else {
                String::new()
            };
            lines.push((curr_addr, format!("{}{:#010x}: {}",label, curr_addr, body)));

            res.extend(lines);

            offset += ins_len;
        }
    }

    res
}

 #[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::disasm::disassembler;
    use riscv_loader::LoadInfo;

    #[test]
    #[cfg(not(feature = "c"))]
    fn test_disassembler_flow() {
        let mut info = LoadInfo::default();
        // 0x00000513 -> addi x10, x0, 0
        // 0x00000073 -> ecall

        let code_bytes = vec![
            0x13, 0x05, 0x00, 0x00, 
            0x73, 0x00, 0x00, 0x00,
        ];
        info.code = vec![(code_bytes, 0x80000000)];
        
        let mut syms = HashMap::new();
        syms.insert(0x80000000, "main".to_string());
        info.symbols = Some(syms);

        let output = disassembler(&info);

        assert!(output[0].1.contains("<main>:"));
        assert!(output[0].1.contains("addi    x10, x0, 0"));
        assert!(output[1].1.contains("ecall"));
    }

    #[test]
    #[cfg(feature = "c")]
    fn test_compress_disassembler() {
        let mut info = LoadInfo::default();
        // 0x717D -> c.addi16sp -16
        // 0x00000073 -> ecall
        // 0x9002 -> c.break

        let code_bytes = vec![
            0x7D, 0x71,
            0x73, 0x00, 0x00, 0x00,
            0x02, 0x90,
        ];
        info.code = vec![(code_bytes, 0x80000000)];
        
        let mut syms = HashMap::new();
        syms.insert(0x80000000, "main".to_string());
        info.symbols = Some(syms);

        let output = disassembler(&info);
        
        assert!(output[0].1.contains("<main>:"));
        assert!(output[0].1.contains("(C) addi    x2, x2, -16"));
        assert!(output[1].1.contains("ecall"));
        assert!(output[2].1.contains("(C) ebreak"));
    }
}
