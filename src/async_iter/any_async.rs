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
struct AnyAsyncFn<P> {
    predicate: P,
}

impl<T, P> FnMut2<(), T> for AnyAsyncFn<P>
where
    P: FnMut1<T>,
{
    type Output = Map<P::Output, BreakIfTrue>;

    fn call_mut(&mut self, (): (), arg_2: T) -> Self::Output {
        Map::new(self.predicate.call_mut(arg_2), BreakIfTrue)
    }
}

pin_project_lite::pin_project! {
    pub struct AnyAsync<I, P>
    where
        I: AsyncIterator,
        P: FnMut1<I::Item>,
    {
        #[pin]
        predicate: Map<TryFoldAsync<I, (), AnyAsyncFn<P>>, ControlFlowIsBreakFn>
    }
}

impl<I, P> AnyAsync<I, P>
where
    I: AsyncIterator,
    P: FnMut1<I::Item>,
{
    pub(crate) fn new(iter: I, predicate: P) -> Self {
        Self {
            predicate: Map::new(
                TryFoldAsync::new(iter, (), AnyAsyncFn { predicate }),
                ControlFlowIsBreakFn::default(),
            ),
        }
    }
}

impl<I, P> Clone for AnyAsync<I, P>
where
    I: AsyncIterator + Clone,
    P: FnMut1<I::Item> + Clone,
    P::Output: Clone,
{
    fn clone(&self) -> Self {
        Self {
            predicate: self.predicate.clone(),
        }
    }
}

impl<I, P> Future for AnyAsync<I, P>
where
    I: AsyncIterator,
    P: FnMut1<I::Item>,
    P::Output: Future<Output = bool>,
{
    type Output = bool;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().predicate.poll(cx)
    }
}

impl<I, P> FusedFuture for AnyAsync<I, P>
where
    I: FusedAsyncIterator,
    P: FnMut1<I::Item>,
    P::Output: FusedFuture<Output = bool>,
{
    fn is_terminated(&self) -> bool {
        self.predicate.is_terminated()
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
        let future = stream::iter([2, 3, 5]).slim_any_async(greater_than_2);

        assert!(future.await);
    }

    #[tokio::test]
    async fn test_any_async_fail() {
        let future = stream::iter([2, 3, 5]).slim_any_async(equals_10);

        assert!(!future.await);
    }

    #[tokio::test]
    async fn test_any_async_clone() {
        let future = stream::iter([2, 3, 5]).slim_any_async(greater_than_2);
        let future_2 = future.clone();

        assert!(future.await);
        assert!(future_2.await);
    }
}
