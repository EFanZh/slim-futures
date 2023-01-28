use crate::async_iter::try_fold_async::TryFoldAsync;
use crate::future::Map;
use crate::support::fns::ControlFlowIsContinueFn;
use crate::support::AsyncIterator;
use core::future::{Future, IntoFuture};
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::fns::CopyFn;
use fn_traits::FnMut;

#[derive(Clone)]
struct ContinueIfTrue;

impl FnMut<(bool,)> for ContinueIfTrue {
    type Output = ControlFlow<()>;

    fn call_mut(&mut self, args: (bool,)) -> Self::Output {
        if args.0 {
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }
}

#[derive(Clone)]
struct AllAsyncFn<P> {
    predicate: P,
}

impl<T, P> FnMut<((), T)> for AllAsyncFn<P>
where
    P: FnMut<(T,)>,
    P::Output: IntoFuture,
{
    type Output = Map<<P::Output as IntoFuture>::IntoFuture, ContinueIfTrue>;

    fn call_mut(&mut self, args: ((), T)) -> Self::Output {
        Map::new(self.predicate.call_mut((args.1,)).into_future(), ContinueIfTrue)
    }
}

pin_project_lite::pin_project! {
    pub struct AllAsync<I, P>
    where
        I: AsyncIterator,
        P: FnMut<(I::Item,)>,
        P::Output: IntoFuture<Output = bool>,
    {
        #[pin]
        inner: Map<TryFoldAsync<I, (), CopyFn, AllAsyncFn<P>>, ControlFlowIsContinueFn>
    }
}

impl<I, P> AllAsync<I, P>
where
    I: AsyncIterator,
    P: FnMut<(I::Item,)>,
    P::Output: IntoFuture<Output = bool>,
{
    pub(crate) fn new(iter: I, predicate: P) -> Self {
        Self {
            inner: Map::new(
                TryFoldAsync::new(iter, (), CopyFn::default(), AllAsyncFn { predicate }),
                ControlFlowIsContinueFn::default(),
            ),
        }
    }
}

impl<I, P> Clone for AllAsync<I, P>
where
    I: AsyncIterator + Clone,
    P: FnMut<(I::Item,)> + Clone,
    P::Output: IntoFuture<Output = bool>,
    <P::Output as IntoFuture>::IntoFuture: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<I, P> Future for AllAsync<I, P>
where
    I: AsyncIterator,
    P: FnMut<(I::Item,)>,
    P::Output: IntoFuture<Output = bool>,
{
    type Output = bool;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iterator_ext::AsyncIteratorExt;
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
