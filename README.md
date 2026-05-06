# mini-os

minimal x86_64 kernel in rust. memory, interrupts, heap, shell.

## architecture

```
bootloader → gdt → idt → pics → paging → heap → shell
```

| module | what it does |
|--------|-------------|
| `gdt` | segment descriptors, tss, user-mode segments |
| `interrupts` | idt — exception handlers, irqs, timer, keyboard |
| `memory` | page tables, frame allocator, page-flag ops |
| `allocator` | heap with 3 strategies: bump, linked list, fixed-size block |
| `vga_buffer` | 80×25 text mode, colored |
| `serial` | uart 16550, test logging |
| `shell` | repl with 10 commands, keyboard input, history |
| `task` | cooperative async executor, task spawning, wakers |
| `ramdisk` | in-memory block device, read/write/hex decode |

## building

needs **rust nightly** (`rust-toolchain.toml`) and qemu.

```bash
cargo install bootimage
cargo bootimage
```

## running

```bash
qemu-system-x86_64 -drive format=raw,file=target/x86_64-mini-os/debug/bootimage-mini-os.bin
```

## testing

custom test framework, runs inside qemu, reports over serial.

```bash
cargo test
```

## license

mit — [LICENSE-MIT](LICENSE-MIT).
