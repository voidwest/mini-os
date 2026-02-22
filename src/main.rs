// main.rs
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
use core::{fmt::write, panic::PanicInfo};
mod vga_buffer;


// called on panic
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { 
    loop{}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> !{
    println!("Hello World{}", "!");
        loop {}
}

#[cfg(test)]