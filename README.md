# miniOS

A minimal x86_64 operating system kernel written in Rust, built following Philipp Oppermann's [Writing an OS in Rust](https://os.phil-opp.com/) series.

## Features

- Boots on bare metal and QEMU
- Memory paging and interrupt handling
- Bare-metal hardware interfaces without the standard library
- Custom linker scripts and bootloader setup
- Network stack *(in progress)*

## Running

### Dependencies

```bash
cargo install bootimage
rustup target add x86_64-unknown-none
```

### Run in QEMU

```bash
cargo run
```

## What I Learned

- Barebones and no std programming.
- Thinking in systems, understanding how the bootloader, kernel, and hardware interact as a whole rather than isolated components.
- Having to know how every piece interacts with one another.
- Writing Rust in a new way using features that usually don't appear.

## References

- [Writing an OS in Rust](https://os.phil-opp.com/) by Philipp Oppermann
