use crate::future::SlimFutureExt;
use futures_core::FusedFuture;
use std::future::Future;
use std::num::NonZeroU32;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Clone)]
pub struct Yield {
    count: Option<usize>,
}

impl Yield {
    pub fn new(count: usize) -> Self {
        Self { count: Some(count) }
    }
}

impl Future for Yield {
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

impl FusedFuture for Yield {
    fn is_terminated(&self) -> bool {
        self.count.is_none()
    }
}

pub fn full_bytes_future(value: u32) -> impl Future<Output = u32> {
    crate::future::ready(value)
}

pub fn almost_full_bytes_future(value: u32) -> impl Future<Output = u32> {
    crate::future::ready(NonZeroU32::new(value).unwrap()).slim_map(NonZeroU32::get)
}
