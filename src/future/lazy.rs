use crate::support;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    pub struct Lazy<F> {
        f: F,
    }
}

impl<F, T> Future for Lazy<F>
where
    F: FnMut(&mut Context) -> T,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        Poll::Ready((self.project().f)(cx))
    }
}

pub fn lazy<F, T>(f: F) -> Lazy<F>
where
    F: FnMut(&mut Context) -> T,
{
    support::assert_future::<_, T>(Lazy { f })
}
