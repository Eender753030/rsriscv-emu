use risc_v_emulator::riscv::RiscV;

fn main() {
    let mut machine = RiscV::new();
    
    let code = 0x02A00093;

    if let Err(e) = machine.load_code(code) {
        eprintln!("Error: {}", e);
    }
    
    if let Err(e) = machine.cycle() {
        eprintln!("Error: {}", e);
    }

    machine.print_registers();
}