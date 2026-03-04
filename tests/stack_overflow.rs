#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
use core::panic::PanicInfo;
use mini_os::{exit_qemu, serial_print, serial_println};
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> !{
    serial_print!("stack_overflow::stack_overflow\t");

    mini_os::gdt::init();
    init_test_idt();

    stack_overflow();
    panic!("execution continued after stack overflow");
}

#[allow(unconditional_recursion)]
fn stack_overflow(){
    let _x: u64 = 0;
    volatile::Volatile::new(&_x).read();
    stack_overflow();
}

lazy_static!{
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe{
            idt.double_fault.set_handler_fn(test_double_fault_handler).set_stack_index(mini_os::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> !{
    serial_println!("[ok]");
    exit_qemu(mini_os::QemuExitCode::Success);
    loop{}
}


pub fn init_test_idt(){
    serial_println!("hey");
    TEST_IDT.load();
    serial_println!("listen");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> !{
    mini_os::test_panic_handler(info)
}