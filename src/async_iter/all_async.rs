use crate::async_iter::try_fold_async::TryFoldAsync;
use crate::future::Map;
use crate::support::fns::ControlFlowIsContinueFn;
use crate::support::{AsyncIterator, FnMut1, FnMut2, FusedAsyncIterator};
use core::future::Future;
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::FusedFuture;

#[derive(Clone)]
struct ContinueIfTrue;

impl FnMut1<bool> for ContinueIfTrue {
    type Output = ControlFlow<()>;

    fn call_mut(&mut self, arg: bool) -> Self::Output {
        if arg {
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }
}

#[derive(Clone)]
struct AllAsyncFn<F> {
    inner: F,
}

impl<T, F> FnMut2<(), T> for AllAsyncFn<F>
where
    F: FnMut1<T>,
{
    type Output = Map<F::Output, ContinueIfTrue>;

    fn call_mut(&mut self, (): (), arg_2: T) -> Self::Output {
        Map::new(self.inner.call_mut(arg_2), ContinueIfTrue)
    }
}

pin_project_lite::pin_project! {
    pub struct AllAsync<I, F>
    where
        I: AsyncIterator,
        F: FnMut1<I::Item>,
    {
        #[pin]
        inner: Map<TryFoldAsync<I, (), AllAsyncFn<F>>, ControlFlowIsContinueFn>
    }
}

impl<I, F> AllAsync<I, F>
where
    I: AsyncIterator,
    F: FnMut1<I::Item>,
{
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self {
            inner: Map::new(
                TryFoldAsync::new(iter, (), AllAsyncFn { inner: f }),
                ControlFlowIsContinueFn::default(),
            ),
        }
    }
}

impl<I, F> Clone for AllAsync<I, F>
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

impl<I, F> Future for AllAsync<I, F>
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

impl<I, F> FusedFuture for AllAsync<I, F>
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

    fn less_than_10(x: u32) -> Ready<bool> {
        future::ready(x < 10)
    }

    fn equals_2(x: u32) -> Ready<bool> {
        future::ready(x == 2)
    }

    #[tokio::test]
    async fn test_all_async() {
        let future = stream::iter([2, 3, 5]).slim_all_async(less_than_10);

        assert!(future.await);
    }

    #[tokio::test]
    async fn test_all_async_fail() {
        let future = stream::iter([2, 3, 5]).slim_all_async(equals_2);

        assert!(!future.await);
    }

    #[tokio::test]
    async fn test_all_async_clone() {
        let future = stream::iter([2, 3, 5]).slim_all_async(less_than_10);
        let future_2 = future.clone();

        assert!(future.await);
        assert!(future_2.await);
    }
}
