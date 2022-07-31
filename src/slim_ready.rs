use crate::assert_future;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct SlimReady<T> {
    value: T,
}

impl<T> Future for SlimReady<T>
where
    T: Copy,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, _: &mut Context) -> Poll<Self::Output> {
        Poll::Ready(self.value)
    }
}

pub fn slim_ready<T>(value: T) -> SlimReady<T>
where
    T: Copy,
{
    assert_future::assert_future::<_, T>(SlimReady { value })
}
