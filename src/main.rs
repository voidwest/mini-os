// main.rs
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(mini_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{BootInfo, entry_point}
use core::panic::PanicInfo;
use mini_os::println;

// called on panic
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use mini_os::memory::active_level_4_table;
    use x86_64::VirtAddr;

    mini_os::init();
    println!("Hello World{}", "!");


    #[cfg(test)]
    test_main();
    println!("didn't crash yet");
    mini_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    mini_os::test_panic_handler(info)
}
