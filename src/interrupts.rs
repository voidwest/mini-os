use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::println;
use lazy_static::lazy_static;





extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame){
println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

