use crate::support;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Clone)]
pub struct Lazy<F> {
    f: F,
}

impl<F> Unpin for Lazy<F> {}

impl<F, T> Future for Lazy<F>
where
    F: FnMut(&mut Context) -> T,
{
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        Poll::Ready((self.f)(cx))
    }
}

pub fn lazy<F, T>(f: F) -> Lazy<F>
where
    F: FnMut(&mut Context) -> T,
{
    support::assert_future::<_, T>(Lazy { f })
}
