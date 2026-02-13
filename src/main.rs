// main.rs
#![no_std]
#![no_main]
use core::panic::PanicInfo;


static HELLO: &[u8] = b"Hello World!";

// called on panic
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { 
    loop{}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> !{
    loop{}
}
