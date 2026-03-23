use alloc::boxed::Box;
use core::{future::Future, pin::Pin};

pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}
