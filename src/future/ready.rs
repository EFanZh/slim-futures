use crate::support;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct Ready<T> {
    value: T,
}

impl<T> Future for Ready<T>
where
    T: Copy,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, _: &mut Context) -> Poll<Self::Output> {
        Poll::Ready(self.value)
    }
}

pub fn ready<T>(value: T) -> Ready<T>
where
    T: Copy,
{
    support::assert_future::<_, T>(Ready { value })
}
