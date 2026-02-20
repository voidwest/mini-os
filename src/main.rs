// main.rs
#![no_std]
#![no_main]
use core::panic::PanicInfo;
mod vga_buffer;


// called on panic
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { 
    loop{}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> !{
    use core::fmt::Write;
    vga_buffer::WRITER.lock().write_str("hi").unwrap();
    loop {}
}

// qemu-system-x86_64 -drive format=raw,file=target/x86_64-mini-os/debug/bootimage-mini-os.bin - irrelevant