use futures_core::FusedFuture;
use std::future::Future;
use std::num::NonZeroUsize;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Clone)]
pub struct Defer {
    count: Option<usize>,
}

impl Defer {
    pub fn new(count: usize) -> Self {
        Self { count: Some(count) }
    }
}

impl Future for Defer {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        if let Some(count) = &mut self.count {
            if *count == 0 {
                self.count = None;

                Poll::Ready(())
            } else {
                *count -= 1;

                cx.waker().wake_by_ref();

                Poll::Pending
            }
        } else {
            Poll::Ready(())
        }
    }
}

impl FusedFuture for Defer {
    fn is_terminated(&self) -> bool {
        self.count.is_none()
    }
}

pub fn full_bytes_future() -> impl Future {
    crate::future::ready(0_usize)
}

pub fn almost_full_bytes_future() -> impl Future {
    crate::future::ready(NonZeroUsize::new(1).unwrap())
}
