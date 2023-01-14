use crate::async_iter::try_fold_async::TryFoldAsync;
use crate::future::Map;
use crate::support::fns::ControlFlowIsBreakFn;
use crate::support::{AsyncIterator, FnMut1, FnMut2, FusedAsyncIterator};
use core::future::Future;
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::FusedFuture;

#[derive(Clone)]
struct BreakIfTrue;

impl FnMut1<bool> for BreakIfTrue {
    type Output = ControlFlow<()>;

    fn call_mut(&mut self, arg: bool) -> Self::Output {
        if arg {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}

#[derive(Clone)]
struct AnyAsyncFn<F> {
    inner: F,
}

impl<T, F> FnMut2<(), T> for AnyAsyncFn<F>
where
    F: FnMut1<T>,
{
    type Output = Map<F::Output, BreakIfTrue>;

    fn call_mut(&mut self, (): (), arg_2: T) -> Self::Output {
        Map::new(self.inner.call_mut(arg_2), BreakIfTrue)
    }
}

pin_project_lite::pin_project! {
    pub struct AnyAsync<I, F>
    where
        I: AsyncIterator,
        F: FnMut1<I::Item>,
    {
        #[pin]
        inner: Map<TryFoldAsync<I, (), AnyAsyncFn<F>>, ControlFlowIsBreakFn>
    }
}

impl<I, F> AnyAsync<I, F>
where
    I: AsyncIterator,
    F: FnMut1<I::Item>,
{
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self {
            inner: Map::new(
                TryFoldAsync::new(iter, (), AnyAsyncFn { inner: f }),
                ControlFlowIsBreakFn::default(),
            ),
        }
    }
}

impl<I, F> Clone for AnyAsync<I, F>
where
    I: AsyncIterator + Clone,
    F: FnMut1<I::Item> + Clone,
    F::Output: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<I, F> Future for AnyAsync<I, F>
where
    I: AsyncIterator,
    F: FnMut1<I::Item>,
    F::Output: Future<Output = bool>,
{
    type Output = bool;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<I, F> FusedFuture for AnyAsync<I, F>
where
    I: FusedAsyncIterator,
    F: FnMut1<I::Item>,
    F::Output: FusedFuture<Output = bool>,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iter_ext::AsyncIteratorExt;
    use futures_util::future::{self, Ready};
    use futures_util::stream;

    fn greater_than_2(x: u32) -> Ready<bool> {
        future::ready(x > 2)
    }

    fn equals_10(x: u32) -> Ready<bool> {
        future::ready(x == 10)
    }

    #[tokio::test]
    async fn test_any_async() {
        let future = stream::iter([2, 3, 5]).any_async(greater_than_2);

        assert!(future.await);
    }

    #[tokio::test]
    async fn test_any_async_fail() {
        let future = stream::iter([2, 3, 5]).any_async(equals_10);

        assert!(!future.await);
    }

    #[tokio::test]
    async fn test_any_async_clone() {
        let future = stream::iter([2, 3, 5]).any_async(greater_than_2);
        let future_2 = future.clone();

        assert!(future.await);
        assert!(future_2.await);
    }
}
