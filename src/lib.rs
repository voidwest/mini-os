#![no_std]

#![cfg_attr(test, no_main)]
#![feature((custom_test_frameworks))]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
pub mod vga_buffer;
pub mod serial;

pub trait Testable {
    fn run(&self) -> ();
}

impl <T> Testable for T 
    where T: Fn(),
    {
        fn run (&self){
            serial_print!("{}...\t", core::any::type_name::<T>());
            self();
            serial_println!("[ok]");
        }
}

pub fn test_runner(tests: &[&dyn Testable]){
    serial_println!("[Failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop{}
}
