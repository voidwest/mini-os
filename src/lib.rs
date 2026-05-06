#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]

use bootloader::BootInfo;
#[cfg(test)]
use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;

extern crate alloc;

pub mod allocator;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod ramdisk;
pub mod serial;
pub mod shell;
pub mod syscall;
pub mod task;
pub mod user;
pub mod vga_buffer;

/// A trait abstracting over testable entities (functions or closures).
///
/// Used by the custom test framework to run test cases and report results.
pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

/// Entry point for the custom test framework.
///
/// Runs all registered tests sequentially, printing results over serial,
/// then exits QEMU with the appropriate exit code.
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

/// Panic handler for test mode. Reports the failure over serial and exits QEMU.
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[Failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[cfg(test)]
entry_point!(test_kernel_main);

#[cfg(test)]
fn test_kernel_main(boot_info: &'static BootInfo) -> ! {
    init_all(boot_info);
    test_main();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

/// Exit codes understood by the `isa-debug-exit` QEMU device.
///
/// Written to port `0xf4` to signal test success or failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

/// Signal QEMU to exit via the `isa-debug-exit` device (port `0xf4`).
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

/// Initialize core kernel subsystems: GDT, IDT, PICs, and enable interrupts.
pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

/// Halt the CPU in a loop. Used as the terminal state when there is nothing
/// left to do and the kernel should idle forever.
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

/// Full kernel initialization: GDT, IDT, PICs, paging, and heap.
///
/// This is the single entry point for setting up all kernel subsystems.
/// Both the main kernel and integration tests should call this.
pub fn init_all(boot_info: &'static BootInfo) {
    use x86_64::VirtAddr;

    init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");
}
