use alloc::boxed::Box;
use alloc::collections::VecDeque;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(0);

/// async task with a unique id, wrapping a pinned future.
pub struct Task {
    id: u64,
    future: Pin<Box<dyn Future<Output = ()> + Send>>,
    woken: AtomicBool,
}

impl Task {
    fn new(future: impl Future<Output = ()> + 'static + Send) -> Self {
        let id = NEXT_TASK_ID.fetch_add(1, Ordering::SeqCst);
        Task { id, future: Box::pin(future), woken: AtomicBool::new(true) }
    }

    fn poll(&mut self, waker: &Waker) -> Poll<()> {
        self.woken.store(false, Ordering::SeqCst);
        let mut cx = Context::from_waker(waker);
        self.future.as_mut().poll(&mut cx)
    }
}

/// cooperative async executor — polls tasks round-robin, re-enqueues on wake.
pub struct Executor {
    tasks: VecDeque<Task>,
    spawn_queue: VecDeque<Task>,
}

impl Executor {
    pub const fn new() -> Self {
        Executor { tasks: VecDeque::new(), spawn_queue: VecDeque::new() }
    }

    /// spawn a new async task.
    pub fn spawn(&mut self, future: impl Future<Output = ()> + 'static + Send) {
        self.spawn_queue.push_back(Task::new(future));
    }

    /// poll tasks for one iteration. drains spawn queue, polls one task.
    /// returns `true` if tasks remain pending.
    pub fn poll_once(&mut self) -> bool {
        // drain spawn queue.
        while let Some(task) = self.spawn_queue.pop_front() {
            self.tasks.push_back(task);
        }

        if self.tasks.is_empty() {
            return false;
        }

        let mut task = self.tasks.pop_front().unwrap();
        let waker = task_waker(task.id);
        match task.poll(&waker) {
            Poll::Ready(()) => {
                crate::serial_println!("[task {}] completed", task.id);
            }
            Poll::Pending => {
                if task.woken.load(Ordering::SeqCst) {
                    self.tasks.push_front(task);
                } else {
                    self.tasks.push_back(task);
                }
            }
        }

        !self.tasks.is_empty()
    }

    /// run until all tasks are complete.
    pub fn run(&mut self) {
        loop {
        // drain the spawn queue.
            while let Some(task) = self.spawn_queue.pop_front() {
                self.tasks.push_back(task);
            }

            if self.tasks.is_empty() {
                // no more tasks — spin with hlt until new tasks arrive.
                // we break and let the caller decide what to do, or just hlt.
                x86_64::instructions::hlt();
                continue;
            }

            // Poll one task.
            let mut task = self.tasks.pop_front().unwrap();
            let waker = task_waker(task.id);
            match task.poll(&waker) {
                Poll::Ready(()) => {
                    crate::serial_println!("[task {}] completed", task.id);
                }
                Poll::Pending => {
                    if task.woken.load(Ordering::SeqCst) {
                        // task was woken while polling — re-queue at front.
                        self.tasks.push_front(task);
                    } else {
                        // not woken — re-queue at back for fairness.
                        self.tasks.push_back(task);
                    }
                }
            }
        }
    }
}

use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    /// global executor instance.
    pub static ref EXECUTOR: Mutex<Executor> = Mutex::new(Executor::new());
}

fn task_waker(task_id: u64) -> Waker {
    unsafe fn clone_raw(data: *const ()) -> RawWaker {
        RawWaker::new(data, &WAKER_VTABLE)
    }
    unsafe fn wake(data: *const ()) {
        let task_id = data as u64;
        // find the task in the executor and move it to the front.
        let mut executor = EXECUTOR.lock();
        if let Some(pos) = executor.tasks.iter().position(|t| t.id == task_id) {
            let task = executor.tasks.remove(pos).unwrap();
            task.woken.store(true, Ordering::SeqCst);
            executor.tasks.push_front(task);
        }
    }
    unsafe fn wake_by_ref(data: *const ()) {
        let task_id = data as u64;
        let executor = EXECUTOR.lock();
        if let Some(task) = executor.tasks.iter().find(|t| t.id == task_id) {
            task.woken.store(true, Ordering::SeqCst);
        }
    }
    unsafe fn drop_raw(_data: *const ()) {}

    static WAKER_VTABLE: RawWakerVTable =
        RawWakerVTable::new(clone_raw, wake, wake_by_ref, drop_raw);

    unsafe { Waker::from_raw(RawWaker::new(task_id as *const (), &WAKER_VTABLE)) }
}

/// run the global executor forever.
pub fn run_executor() -> ! {
    loop {
        EXECUTOR.lock().run();
    }
}

/// spawn a task on the global executor.
pub fn spawn(future: impl Future<Output = ()> + 'static + Send) {
    EXECUTOR.lock().spawn(future);
}

/// poll the global executor for one iteration. returns `true` if tasks remain.
pub fn poll_once() -> bool {
    EXECUTOR.lock().poll_once()
}
