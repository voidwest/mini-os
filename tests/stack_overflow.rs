#![no_std]
#![no_main]
use core::panic::PanicInfo;
use mini_os::{serial_print, serial_println};
use lazy_static::lazy_static;
use x86_64::structures::idt::InterruptDescriptorTable;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> !{
    serial_print!("stack_overflow::stack_overflow\t");

    mini_os::gdt::init();
    init_test_idt();

    stack_overflow();
    panic!("execution continued after stack overflow");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> !{
    mini_os::test_panic_handler(info)
}

#[allow(unconditional_recursion)]
fn stack_overflow(){
    stack_overflow();
    volatile::Volatile::new(0).read();
}

