#![no_std]
#![no_main]

use core::panic::PanicInfo;
use mini_os::{QemuExitCode, exit_qemu, serial_println};

pub extern "C" fn _start() -> !{
    should_fail();
    serial_println!("[did not panic]");
    exit_qemu(QemuExitCode::Failed);
    loop{}
}

fn should_fail(){
    serial_println!("should_panic should fail....\t");
    assert_eq!(0, 1);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> !{
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop{}
}