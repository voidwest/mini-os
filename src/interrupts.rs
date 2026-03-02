use x86_64::structures::idt::InterruptDescriptorTable;
use crate::println;

pub fn init_idt(){
    let mut idt = InterruptDescriptorTable::new();
    idt.breakpoint.set_handler_fn(breakpoint_handler);
}

