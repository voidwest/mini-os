use x86_64::VirtAddr;

use core::arch::global_asm;

global_asm!(
    ".global user_program",
    ".type user_program, @function",
    "user_program:",
    // SYS_WRITE: print a message.
    "mov rax, 1",
    "lea rdi, [rip + user_msg]",
    "mov rsi, 22",
    "int 0x80",
    // SYS_EXIT: terminate.
    "mov rax, 2",
    "int 0x80",
    "ud2",
    "user_msg:",
    ".ascii \"hello from user mode!\\n\"",
);

unsafe extern "sysv64" {
    fn user_program();
}

/// enter ring 3 (user mode) and execute the embedded user program.
///
/// when the user program calls SYS_EXIT, control returns to `kernel_resume`.
///
/// # Safety
/// gdt, tss, idt, and paging must be initialized before calling.
pub unsafe fn enter_user_mode() -> ! {
    let (user_code_sel, user_data_sel) = crate::gdt::user_selectors();
    let user_code_addr = VirtAddr::new(user_program as *const () as u64);

    // mark the page containing the user program as user-accessible.
    unsafe { crate::memory::mark_page_user(user_code_addr) };

    // allocate a user stack on the heap.
    let user_stack = alloc::vec![0u8; 4096];
    let user_stack_top = VirtAddr::new(user_stack.as_ptr() as u64 + 4096);

    // set up the exit context so that SYS_EXIT returns to kernel_resume.
    unsafe {
        let stack_top = core::ptr::addr_of!(KERNEL_EXIT_STACK) as u64 + 4096;
        crate::syscall::set_exit_context(stack_top, kernel_resume as *const () as u64);
    }

    // prevent the stack Vec from being dropped.
    core::mem::forget(user_stack);

    unsafe {
        core::arch::asm!(
            "push {data_sel}",
            "push {user_stack}",
            "pushfq",
            "push {code_sel}",
            "push {user_code}",
            "iretq",
            data_sel = in(reg) user_data_sel.0 as u64,
            user_stack = in(reg) user_stack_top.as_u64(),
            code_sel = in(reg) user_code_sel.0 as u64,
            user_code = in(reg) user_code_addr.as_u64(),
            options(nomem, nostack),
        );
    }
    unreachable!();
}

/// kernel stack used when returning from user mode via SYS_EXIT.
static mut KERNEL_EXIT_STACK: [u8; 4096] = [0; 4096];

/// resume point called by the syscall exit path. re-enters the shell.
extern "sysv64" fn kernel_resume() -> ! {
    crate::shell::run();
}
