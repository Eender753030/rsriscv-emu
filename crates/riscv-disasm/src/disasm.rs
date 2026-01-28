use std::collections::HashMap;

use riscv_loader::LoadInfo;
use riscv_decoder::decoder::decode;

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

 #[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::disasm::disassembler;
    use riscv_loader::LoadInfo;

    #[test]
    fn test_disassembler_flow() {
        let mut info = LoadInfo::default();
        // 0x00000513 -> addi x10, x0, 0
        // 0x00000073 -> ecall
        // 0x12000073 -> sfence.vma x0, x0
        let code_bytes = vec![
            0x13, 0x05, 0x00, 0x00, 
            0x73, 0x00, 0x00, 0x00,
            0x73, 0x00, 0x00, 0x12,
        ];
        info.code = vec![(code_bytes, 0x80000000)];
        
        let mut syms = HashMap::new();
        syms.insert(0x80000000, "main".to_string());
        info.symbols = Some(syms);

        let output = disassembler(&info);

        assert!(output.contains(&"main:".to_string()));
        assert!(output[1].contains("addi    x10, x0, 0"));
        assert!(output[2].contains("ecall"));
        assert!(output[3].contains("sfence.vma   x0, x0"));
    }
}
