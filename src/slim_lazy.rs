use crate::assert_future;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    pub struct SlimLazy<F> {
        f: F,
    }
}

impl<F, T> Future for SlimLazy<F>
where
    F: FnMut(&mut Context) -> T,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        Poll::Ready((self.project().f)(cx))
    }
}

pub fn slim_lazy<F, T>(f: F) -> SlimLazy<F>
where
    F: FnMut(&mut Context) -> T,
{
    assert_future::assert_future::<_, T>(SlimLazy { f })
}
