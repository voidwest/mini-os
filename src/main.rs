// main.rs
#![no_std]
use core::panic::PanicInfo;

fn main(){}


// called on panic
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { 
    loop{}
}