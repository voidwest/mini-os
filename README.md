# mini-os

A minimal x86_64 operating system kernel written in Rust, featuring memory
management, interrupt handling, heap allocation, and an interactive shell.

## Architecture

```
bootloader → GDT → IDT → PICs → Paging → Heap → Shell
```

| Module | Description |
|--------|-------------|
| `gdt` | Global Descriptor Table — segment descriptors, TSS, user-mode segments |
| `interrupts` | Interrupt Descriptor Table — exception handlers, IRQs, syscall vector |
| `memory` | Page-table walking, frame allocation, page-flag manipulation |
| `allocator` | Heap initialization + 3 allocator strategies (bump, linked list, fixed-size block) |
| `vga_buffer` | VGA text-mode output (80×25, colored) |
| `serial` | UART 16550 serial output for test logging |
| `shell` | Interactive REPL with keyboard input and command dispatch |
| `task` | Cooperative async executor with task spawning and wakers |
| `ramdisk` | Simple in-memory block device with read/write commands |
| `syscall` | Assembly syscall handler (int 0x80) with dispatch logic |
| `user` | Ring 3 entry point and embedded user-mode program |

## Features

- [x] VGA text-mode output with color support
- [x] Serial port output (UART 16550)
- [x] GDT with TSS (double-fault IST)
- [x] IDT with exception handlers (breakpoint, double fault, page fault)
- [x] PIC-based interrupt handling (timer, PS/2 keyboard)
- [x] Paging and physical memory frame allocation
- [x] Heap allocation (fixed-size block allocator with fallback)
- [x] Keyboard input (PS/2 scancode decoding)
- [x] Interactive shell / REPL with command dispatch
- [x] Cooperative async executor with task spawning
- [x] Ramdisk / in-memory block device
- [x] Syscall interface with user mode (ring 3)
- [x] Custom test framework running in QEMU

## Building

Requires **Rust nightly** (see `rust-toolchain.toml`) and QEMU.

```bash
# Install the bootimage tool
cargo install bootimage

# Build the kernel image
cargo bootimage
```

## Running

```bash
# Run in QEMU
qemu-system-x86_64 -drive format=raw,file=target/x86_64-mini-os/debug/bootimage-mini-os.bin
```

Or use `cargo run` if the bootimage runner is configured.

## Testing

The project uses a custom test framework that runs inside QEMU and reports
results over the serial port.

```bash
cargo test
```

Tests cover: VGA output, heap allocation, interrupt handling, panic behavior,
and stack overflow via double-fault handling.

## License

MIT — see [LICENSE-MIT](LICENSE-MIT).
