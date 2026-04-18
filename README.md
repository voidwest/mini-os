# mini-os

a minimal x86_64 kernel written in rust. built to handle the low-level handshake between hardware and software without an underlying operating system.

## features
- boots on bare metal + qemu
- memory paging & interrupt handling
- bare-metal hardware interfaces (no-std)
- custom linker scripts & bootloader setup
- network stack (wip)

## tech stack
- rust (nightly)
- llvm (x86_64-unknown-none)
- qemu (emulation)
- bootimage (bootloader integration)

---

## design decisions

### why no_std?
to run on bare metal, the kernel cannot depend on the rust standard library, which requires an existing os (libc). writing in `no_std` ensures the binary only contains code that can execute directly on the cpu.

### why a custom target spec?
instead of using a generic target, a custom `x86_64-mini-os.json` was implemented to define the exact cpu features, memory layout, and disabling of features like red zones. this allows for full control over how the llvm compiler generates the kernel binary.

### why custom linker scripts?
the bootloader expects the kernel entry point to be at a specific memory address. the custom linker setup ensures that the kernel sections (text, data, rodata) are mapped correctly in the executable, preventing memory corruption at boot.

### handling interrupts
used a custom global descriptor table (gdt) and interrupt descriptor table (idt) to handle hardware interrupts and exceptions (like double faults) safely in rust without crashing the system.

---

## getting started

### 1. deps
```bash
rustup target add x86_64-unknown-none
cargo install bootimage
```

### 2. run
```bash
cargo run
```
## roadmap

[ ] full network stack implementation

[ ] basic multitasking / user-mode threads

[ ] simple filesystem support

[ ] graphical output support

## references

### Writing an OS in Rust by Philipp Oppermann

license

mit
