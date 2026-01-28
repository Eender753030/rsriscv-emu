# Changelog

## [Unreleased]
### Added
- **TLB**: Added TLB for MMU. Use P-LRU for victim algorithm. And added the `sfence.vma` instruction to flush TLB.
- **PMP**: Added Physical Memory Protection (PMP) that check between MMU and Bus
- **TUI: Exception Panel**: Added exception panel that shows last exception which be raised by CPU

### Changed
- Refactor `riscv-tui` project architecture

### Fixed
- Fixed panic when PC not in instructions' scope

## [0.2.1] - 2026-01-26
### Added
- **Unit Tests**: Added unit tests for important components.
- **RiscV Tests**: Added intergration test by using offical `riscv-tests`'s `isa/` that include `rv32ui-p`, `rv32um-p`, `rv32mi-p` 

## [0.2.0] - 2026-01-25
### Added
- **MMU Support**: Implemented Sv32 virtual memory translation and Page Tables.
- **Disassembler**: New `riscv-disasm` crate for converting instructions to strings.
- **ELF Loader**: Enhanced loader to support `.elf` headers and symbol names.
- **Exceptions**: Added comprehensive trap handling (Page Fault, Access Fault).
- **TUI**: Integrated disassembler view to show instruction mnemonics.

### Changed
- **Breaking**: Refactored `Bus` and `Device` traits to support `Access` structure (required for MMU).
- **Breaking**: Memory access logic now goes through MMU translation.
- Refactored project structure into a Cargo Workspace (`core`, `decoder`, `loader`, `apps`).
- Optimized `riscv-decoder` for static compiler optimizations.

### Fixed
- Fixed panic on unknown CSR access.
- Fixed incorrect sign extension in immediate values.
- Fixed logic errors in `ecall` exception handling from User/Supervisor modes.

## [0.1.0] - 2025-12-26
### Added
- Initial release of **RsRiscV Emulator**.
- Basic RV32I instruction set support.
- Simple TUI interface.
- Basic UART output support.