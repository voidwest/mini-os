use core::arch::global_asm;

global_asm!(
    ".global syscall_handler_asm",
    ".type syscall_handler_asm, @function",
    "syscall_handler_asm:",
    // Save caller-saved registers.
    "push rdi",
    "push rsi",
    "push rdx",
    "push rcx",
    "push r8",
    "push r9",
    "push r10",
    "push r11",
    // Call handle_syscall(num=rax, arg1=rdi_orig, arg2=rsi_orig) -> ret in rax.
    "mov rdi, rax",
    "mov rsi, [rsp+56]",
    "mov rdx, [rsp+48]",
    "call handle_syscall",
    // Check if this was an exit syscall (return value == 1).
    "cmp rax, 1",
    "je syscall_exit_kernel",
    // Normal return: restore registers and iretq back to user mode.
    "pop r11",
    "pop r10",
    "pop r9",
    "pop r8",
    "pop rcx",
    "pop rdx",
    "pop rsi",
    "pop rdi",
    "iretq",
    "syscall_exit_kernel:",
    // Discard saved registers and iretq frame, then resume kernel shell.
    "add rsp, 64",
    "add rsp, 40",
    // Switch to kernel stack and return to the shell resume point.
    "mov rsp, QWORD PTR [rip + exit_kernel_stack]",
    "mov rax, QWORD PTR [rip + exit_kernel_rip]",
    "jmp rax",
    ".global exit_kernel_stack",
    ".global exit_kernel_rip",
    "exit_kernel_stack:",
    ".quad 0",
    "exit_kernel_rip:",
    ".quad 0",
);

unsafe extern "sysv64" {
    fn syscall_handler_asm();
}

pub const SYS_WRITE: u64 = 1;
pub const SYS_EXIT: u64 = 2;

/// Set the kernel stack pointer and return address to use after SYS_EXIT.
///
/// # Safety
/// Must point to valid kernel stack and a function that never returns.
pub unsafe fn set_exit_context(stack_ptr: u64, rip: u64) {
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(exit_kernel_stack), stack_ptr);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(exit_kernel_rip), rip);
    }
}

// Declared in global_asm above.
unsafe extern "Rust" {
    static mut exit_kernel_stack: u64;
    static mut exit_kernel_rip: u64;
}

/// Syscall dispatch: called from the assembly stub.
#[unsafe(no_mangle)]
extern "sysv64" fn handle_syscall(num: u64, ptr: u64, len: u64) -> u64 {
    match num {
        SYS_WRITE => {
            let slice = unsafe { core::slice::from_raw_parts(ptr as *const u8, len as usize) };
            if let Ok(s) = core::str::from_utf8(slice) {
                crate::println!("{}", s);
                0
            } else {
                u64::MAX
            }
        }
        SYS_EXIT => {
            crate::println!("[user program exited]");
            1 // Signal assembly to return to kernel
        }
        _ => u64::MAX,
    }
}

/// Get a raw function pointer to the syscall handler.
pub fn handler_addr() -> u64 {
    syscall_handler_asm as *const () as u64
}
