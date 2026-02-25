// main.rs
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::{fmt::write, panic::PanicInfo};
mod vga_buffer;
mod serial;

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where 
    T: Fn(),
    {
        fn run(&self) {
            serial_println!("{}...\t", core::any::type_name::<T>());
            self();
            serial_println!("[ok]");
        }
    }

// called on panic
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop{}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> !{
    println!("Hello World{}", "!");

    #[cfg(test)]
    test_main();
        loop {}
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failure = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode){
    use x86_64::instructions::port::Port;

    unsafe{
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}


#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> !{
    serial_println!("[test failed\n");
    serial_println!("error: {}\n", info);
    exit_qemu(QemuExitCode::Failure);
    loop{}
}

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]){
    serial_println!("running {} tests", tests.len());
    for test in tests{
        test();
    }
    exit_qemu(QemuExitCode::Success);
}

#[test_case]
fn ezpz(){
    serial_print!("ezpz test");
    assert_eq!(1, 1);
    serial_println!(" [ok]");
}