use alloc::string::String;
use alloc::vec::Vec;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    static ref INPUT_QUEUE: Mutex<Vec<char>> = Mutex::new(Vec::new());
}

/// Push a character into the shell input queue. Called from the keyboard
/// interrupt handler.
pub fn push_char(c: char) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        INPUT_QUEUE.lock().push(c);
    });
}

/// Pop a character from the input queue if one is available.
fn read_char() -> Option<char> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut queue = INPUT_QUEUE.lock();
        if queue.is_empty() { None } else { Some(queue.remove(0)) }
    })
}

/// Read a line of input, echoing characters and handling backspace.
fn read_line() -> String {
    let mut buf = String::new();
    loop {
        if let Some(c) = read_char() {
            match c {
                '\n' | '\r' => {
                    crate::println!();
                    return buf;
                }
                '\x08' => {
                    // Backspace: remove last character and echo backspace sequence.
                    if !buf.is_empty() {
                        buf.pop();
                        crate::print!("\x08 \x08");
                    }
                }
                c if c.is_ascii_graphic() || c == ' ' => {
                    buf.push(c);
                    crate::print!("{}", c);
                }
                _ => {}
            }
        } else {
            x86_64::instructions::hlt();
        }
    }
}

/// Dispatch a command line by splitting on whitespace and matching the first
/// token against known commands.
fn dispatch(line: &str) {
    let mut parts = line.split_whitespace();
    let cmd = match parts.next() {
        Some(c) => c,
        None => return,
    };

    match cmd {
        "help" => {
            crate::println!("available commands:");
            crate::println!("  help        show this message");
            crate::println!("  echo <msg>  print a message");
            crate::println!("  alloc       run heap allocation demo");
            crate::println!("  clear       clear the screen");
            crate::println!("  meminfo     print heap info");
            crate::println!("  spawn <n>   spawn n async counter tasks");
            crate::println!("  exec        run async executor polling");
            crate::println!("  rdread <offset> <len>  read from ramdisk");
            crate::println!("  rdwrite <offset> <hex> write bytes to ramdisk");
            crate::println!("  rdsize      print ramdisk size");
            crate::println!("  usermode    run a program in ring 3");
        }
        "echo" => {
            let msg = parts.collect::<Vec<&str>>().join(" ");
            crate::println!("{}", msg);
        }
        "alloc" => cmd_alloc(),
        "clear" => cmd_clear(),
        "meminfo" => cmd_meminfo(),
        "spawn" => {
            let arg = parts.next().unwrap_or("");
            cmd_spawn(arg);
        }
        "exec" => cmd_exec(),
        "rdread" => cmd_rdread(parts.next().unwrap_or(""), parts.next().unwrap_or("")),
        "rdwrite" => cmd_rdwrite(parts.next().unwrap_or(""), parts.next().unwrap_or("")),
        "rdsize" => cmd_rdsize(),
        "usermode" => cmd_usermode(),
        "" => {}
        other => crate::println!("unknown command: {}\ntype 'help' for available commands", other),
    }
}

/// Demonstration: allocate and drop heap values to exercise the allocator.
fn cmd_alloc() {
    use alloc::boxed::Box;
    use alloc::vec::Vec;

    let heap_value = Box::new(42);
    crate::println!("allocated Box(42) at {:p}", heap_value);

    let mut v = Vec::new();
    for i in 0..10 {
        v.push(i);
    }
    crate::println!("allocated Vec of {} elements at {:p}", v.len(), v.as_slice());
    crate::println!("sum of elements: {}", v.iter().sum::<i32>());
}

/// Clear the VGA text-mode screen by scrolling all content off.
fn cmd_clear() {
    for _ in 0..25 {
        crate::println!();
    }
}

/// Print basic heap information.
fn cmd_meminfo() {
    crate::println!(
        "heap: start={:#x}, size={} KiB",
        crate::allocator::HEAP_START,
        crate::allocator::HEAP_SIZE / 1024
    );
}

/// Spawn N async counter tasks that print a message after completing.
fn cmd_spawn(args: &str) {
    let n: u32 = match args.parse().ok() {
        Some(n) if n > 0 => n,
        _ => {
            crate::println!("usage: spawn <n>  (n > 0)");
            return;
        }
    };

    for i in 0..n {
        crate::task::spawn(counter_task(i));
    }
    crate::println!("spawned {} tasks", n);
}

async fn counter_task(id: u32) {
    crate::println!("[task {}] started", id);
    // Yield a few times to simulate work.
    for _ in 0..3 {
        yield_now().await;
    }
    crate::println!("[task {}] done", id);
}

/// A simple future that yields once, returning `Pending` and then `Ready` on
/// the next poll.
struct YieldNow {
    yielded: bool,
}

impl Future for YieldNow {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.yielded {
            Poll::Ready(())
        } else {
            self.yielded = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

fn yield_now() -> YieldNow {
    YieldNow { yielded: false }
}

/// Run the executor in a busy loop until all tasks complete, or the user
/// presses a key (Esc to abort). In practice, runs for a limited number of
/// polls since tasks are short-lived.
fn cmd_exec() {
    let mut polls = 0;
    loop {
        if !crate::task::poll_once() {
            crate::println!("all tasks completed ({} polls)", polls);
            break;
        }
        polls += 1;

        // Allow aborting by checking for an Escape key.
        if let Some('\x1b') = read_char_nonblock() {
            crate::println!("\naborted after {} polls", polls);
            break;
        }
    }
}

fn read_char_nonblock() -> Option<char> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut queue = INPUT_QUEUE.lock();
        if queue.is_empty() { None } else { Some(queue.remove(0)) }
    })
}

/// Read from the ramdisk and print the contents as hex.
fn cmd_rdread(offset_str: &str, len_str: &str) {
    let offset: usize = match offset_str.parse() {
        Ok(v) => v,
        Err(_) => {
            crate::println!("usage: rdread <offset> <len>");
            return;
        }
    };
    let len: usize = match len_str.parse() {
        Ok(v) if v > 0 => v,
        _ => {
            crate::println!("usage: rdread <offset> <len>  (len > 0)");
            return;
        }
    };

    let mut buf = alloc::vec![0u8; len];
    match crate::ramdisk::read(offset, &mut buf) {
        Ok(n) => {
            crate::println!("read {} bytes at offset {}:", n, offset);
            for chunk in buf[..n].chunks(16) {
                for byte in chunk {
                    crate::print!("{:02x} ", byte);
                }
                crate::println!();
            }
        }
        Err(e) => crate::println!("error: {}", e),
    }
}

/// Write hex bytes to the ramdisk.
fn cmd_rdwrite(offset_str: &str, hex_str: &str) {
    let offset: usize = match offset_str.parse() {
        Ok(v) => v,
        Err(_) => {
            crate::println!("usage: rdwrite <offset> <hex_bytes>");
            return;
        }
    };
    if hex_str.is_empty() {
        crate::println!("usage: rdwrite <offset> <hex_bytes>");
        return;
    }

    let bytes = match hex::decode(hex_str) {
        Ok(b) => b,
        Err(_) => {
            crate::println!("invalid hex string");
            return;
        }
    };

    match crate::ramdisk::write(offset, &bytes) {
        Ok(n) => crate::println!("wrote {} bytes at offset {}", n, offset),
        Err(e) => crate::println!("error: {}", e),
    }
}

/// Print ramdisk information.
fn cmd_rdsize() {
    crate::println!(
        "ramdisk: {} bytes ({} KiB)",
        crate::ramdisk::size(),
        crate::ramdisk::size() / 1024
    );
}

/// Decode a hex string into bytes.
mod hex {
    pub fn decode(s: &str) -> Result<alloc::vec::Vec<u8>, ()> {
        let s = s.trim();
        if s.len() % 2 != 0 {
            return Err(());
        }
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| ()))
            .collect()
    }
}

/// Enter ring 3 (user mode) and run the embedded user program.
/// Control returns to the shell when the program calls SYS_EXIT.
fn cmd_usermode() {
    crate::println!("entering user mode...");
    unsafe { crate::user::enter_user_mode() };
}

/// Enter the shell's interactive read-eval-print loop.
pub fn run() -> ! {
    crate::println!("mini-os shell — type 'help' for commands");
    loop {
        crate::print!("> ");
        let line = read_line();
        dispatch(&line);
    }
}
