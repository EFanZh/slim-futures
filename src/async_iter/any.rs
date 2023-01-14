use crate::async_iter::try_fold::TryFold;
use crate::future::Map;
use crate::support::fns::ControlFlowIsBreakFn;
use crate::support::{AsyncIterator, FnMut1, FnMut2, FusedAsyncIterator};
use core::future::Future;
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::FusedFuture;

#[derive(Clone)]
struct AnyFn<F> {
    inner: F,
}

impl<T, F> FnMut2<(), T> for AnyFn<F>
where
    F: FnMut1<T, Output = bool>,
{
    type Output = ControlFlow<()>;

    fn call_mut(&mut self, (): (), arg_2: T) -> Self::Output {
        if self.inner.call_mut(arg_2) {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}

pin_project_lite::pin_project! {
    pub struct Any<I, F> {
        #[pin]
        inner: Map<TryFold<I, (), AnyFn<F>>, ControlFlowIsBreakFn>
    }
}

impl<I, F> Any<I, F> {
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self {
            inner: Map::new(
                TryFold::new(iter, (), AnyFn { inner: f }),
                ControlFlowIsBreakFn::default(),
            ),
        }
    }
}

impl<I, F> Clone for Any<I, F>
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

impl<I, F> Future for Any<I, F>
where
    I: AsyncIterator,
    F: FnMut1<I::Item, Output = bool>,
{
    type Output = bool;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<I, F> FusedFuture for Any<I, F>
where
    I: FusedAsyncIterator,
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

    fn greater_than_2(x: u32) -> bool {
        x > 2
    }

    fn equals_10(x: u32) -> bool {
        x == 10
    }

    #[tokio::test]
    async fn test_any() {
        let future = stream::iter([2, 3, 5]).any(greater_than_2);

        assert!(future.await);
    }

    #[tokio::test]
    async fn test_any_fail() {
        let future = stream::iter([2, 3, 5]).any(equals_10);

        assert!(!future.await);
    }

    #[tokio::test]
    async fn test_any_clone() {
        let future = stream::iter([2, 3, 5]).any(greater_than_2);
        let future_2 = future.clone();

        assert!(future.await);
        assert!(future_2.await);
    }
}
