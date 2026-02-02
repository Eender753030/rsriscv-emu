![Language](https://img.shields.io/badge/language-Rust-orange.svg) ![License](https://img.shields.io/badge/license-MIT-blue.svg) ![Backend](https://img.shields.io/badge/UI-Ratatui-green.svg)
# RsRiscV Emu: Interactive RV32IMAC Emulator in Rust

(Not update for now) [RsRiscV Asm here](https://github.com/Eender753030/RsRiscV_asm)

## Key Features
- **ISA Support**:
    - **RV32IMAC Core**: Implements Base Integer (I), Multiply/Divide (M), Atomic (A), and Compressed (C) extensions.
    - **Standard Extensions**: Supports **Zicsr** (Control and Status Register) and **Zifencei**.
    - **Privileged Mode**: Implements **Machine Mode (M-Mode)** with precise Exception.
    - **Memory Management (MMU)**: Full **Sv32** Virtual Memory support with Translation Lookaside Buffer (TLB) and Page Table checking.
    - **Compliance**: Passes official **[riscv-tests](https://github.com/riscv-software-src/riscv-tests)** suites:
        - `rv32ui-p` (User Integer)
        - `rv32um-p` (User Multiply)
        - `rv32ua-p`
        - `rv32uc-p` 
        - Some `rv32si-p`
        
    
- **System & Architecture**:
    - **Modular Design**: Built as a Cargo Workspace separating `core` logic, `decoder`, `disasm`, `loader`, and `tui`.
    - **Feature Flags As Extensions**: Using features flags to simulate adding extension to the CPU.
    - **Memory**: **2GB** Virtualized/Demand-Paged DRAM (base address `0x8000_0000`).
    - **UART**: Memory-mapped serial output at `0x1000_0000` (mapped to host stdout).
    - **Exceptions**: Comprehensive trap handling including Page Faults, Access Faults, and Illegal Instructions.

- **File Loader**:
    - **ELF Support**: Automatically parses ELF headers, loads segments (text/data), and initializes BSS.
    - **Raw Binary**: Fallback support for flat binary files.

- **Interactive TUI**:
    - **Live Disassembly**: Real-time instruction decoding and pipeline visualization.
    - **Dual Register View**: Toggle between **General Purpose Registers (x0-x31)** and **CSRs** (mstatus, mepc, etc.).
    - **(new) Information Popup**: Basic machine information and data.
    - **(new) Bus Content View:**: Search bus bytes with address.
    - **(new) Breakpoint:**: Run until encounter break point. Can have multiple breakpoints.
    - **Exception Panel**: See the exception and its code with raised address.

## Demo
![RsRisc-V Demo](./assets/v0.4.0_demo.gif)

## Quick Start
### Prerequisites
- Rust toolchain (stable) installed.

### Installation

```bash
# Clone the repository
git clone https://github.com/Eender753030/RsRiscV_emu.git
cd RsRiscV_emu

# Build the project in release mode for best performance
cargo build --release

# Add extensions by using feature flags
cargo build --release --features "m,a,c"
```
### Usage
Run the emulator by passing a ELF file as an argument:

```Bash

# Syntax
cargo run --release <path_to_ELF_file>

# Example
cargo run --release ./test
# or
cargo run --release --all-features -- ./test
```
**Note**: The input file can be a standard **ELF** file or a raw binary (Little Endian).

## Controls & Key Bindings

The UI is designed to be keyboard-centric for efficiency.

| Context | Key | Action | Description |
| :--- | :--- | :--- | :--- |
| **General** | `Tab` | **Change Mode** | Toggle between **Observation** (Browse) and **Emulate** (Debug) modes. |
|             | `Q` | **Quit** | Exit the application immediately. |
|             | `I` | **Information Popup** | Show a popup that contain DRAM's size, base, page size, TLB hit rate, Current privileged mode, and current PC. |
|             | `↑` / `↓` | **Scroll** | Scroll through the lists in the currently active panel. |
|             | `C` | **Toggle View** | Switch between **GPR (x0-x31)** and **CSR** view in the Register/CSR panel. |
|             | `H` | **Decimal/Hex** | Switch betwenn **Decimal** and **Hex** of data display in Register/CSR panel. |
| **Navigation**<br>*(Observation)* | `←` / `→` | **Change Panel** | Move focus between Instruction and Register/CSR |
| | `B` | **Breakpoint** | Set/Remove breakpoint on seleted instruction. |
| | `V` | **Bus Search** | Search by enter hex address. If valid, show a popup content 68 bytes start from entered address. |
| **Debug**<br>*(Emulate)* | `S` | **Step** | Execute the next instruction (Single-step). |
| | `P` | **Run to End** | Continuously execute instructions until program exit or error. |
| | `R` | **Reset** | Reset PC to initial state and clear registers/memory. |

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
