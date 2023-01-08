use crate::async_iter::TryFold;
use crate::support::{AsyncIterator, FnMut1, FnMut2};
use core::task;
use futures_core::{FusedFuture, FusedStream};
use std::future::Future;
use std::ops::ControlFlow;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Clone)]
struct AllFn<F> {
    inner: F,
}

impl<T, F> FnMut2<(), T> for AllFn<F>
where
    F: FnMut1<T, Output = bool>,
{
    type Output = ControlFlow<()>;

    fn call_mut(&mut self, (): (), arg_2: T) -> Self::Output {
        if self.inner.call_mut(arg_2) {
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }
}

pin_project_lite::pin_project! {
    pub struct All<I, F> {
        #[pin]
        inner: TryFold<I, (), AllFn<F>>
    }
}

impl<I, F> All<I, F> {
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self {
            inner: TryFold::new(iter, (), AllFn { inner: f }),
        }
    }
}

impl<I, F> Clone for All<I, F>
where
    I: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<I, F> Future for All<I, F>
where
    I: AsyncIterator,
    F: FnMut1<I::Item, Output = bool>,
{
    type Output = bool;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        Poll::Ready(task::ready!(self.project().inner.poll(cx)).is_continue())
    }
}

impl<I, F> FusedFuture for All<I, F>
where
    I: FusedStream,
    F: FnMut1<I::Item, Output = bool>,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iter_ext::AsyncIteratorExt;
    use futures_util::stream;

    #[tokio::test]
    async fn test_all() {
        let future = stream::iter([2, 3, 5]).all(|x| x < 10);

        assert!(future.await);
    }

    #[tokio::test]
    async fn test_all_fail() {
        let future = stream::iter([2, 3, 5]).all(|x| x == 2);

        assert!(!future.await);
    }

    #[tokio::test]
    async fn test_all_clone() {
        let future = stream::iter([2, 3, 5]).all(|x| x < 10);
        let future_2 = future.clone();

        assert!(future.await);
        assert!(future_2.await);
    }
}
