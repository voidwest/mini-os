// main.rs
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(mini_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};
use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;
use mini_os::{memory, println};
// called on panic
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use mini_os::allocator;
    use mini_os::memory::BootInfoFrameAllocator;
    use x86_64::VirtAddr;

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);

    let mut mapper = unsafe { memory::init(phys_mem_offset) };

    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap init failed");

    let heap_value = Box::new(41);
    println!("heap value at {:p}", heap_value);

    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();

    println!(
        "current reference count is {}",
        Rc::strong_count(&cloned_reference)
    );
    core::mem::drop(reference_counted);
    println!(
        "dropped reference count is {}",
        Rc::strong_count(&cloned_reference)
    );

    mini_os::init();
    println!("Hello World{}", "!");

    #[cfg(test)]
    test_main();
    println!("didn't crash yet");
    mini_os::hlt_loop();
}

async fn async_num() -> u8 {
    42
}

async fn example_task() {
    let number = async_num().await;
    println!("async num is {}", number);
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    mini_os::test_panic_handler(info)
}
