use crate::async_iter::try_fold_async::TryFoldAsync;
use crate::future::Map;
use crate::support::AsyncIterator;
use core::future::{Future, IntoFuture};
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{self, Context, Poll};
use fn_traits::fns::CopyFn;
use fn_traits::FnMut;

#[derive(Clone)]
struct BreakIfTrue;

impl FnMut<(bool,)> for BreakIfTrue {
    type Output = ControlFlow<()>;

    fn call_mut(&mut self, args: (bool,)) -> Self::Output {
        if args.0 {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}

#[derive(Clone)]
struct AnyAsyncFn<P>
where
    P: ?Sized,
{
    predicate: P,
}

impl<T, P> FnMut<((), T)> for AnyAsyncFn<P>
where
    P: FnMut<(T,)> + ?Sized,
    P::Output: IntoFuture,
{
    type Output = Map<<P::Output as IntoFuture>::IntoFuture, BreakIfTrue>;

    fn call_mut(&mut self, args: ((), T)) -> Self::Output {
        Map::new(self.predicate.call_mut((args.1,)).into_future(), BreakIfTrue)
    }
}

pin_project_lite::pin_project! {
    pub struct AnyAsync<I, P>
    where
        I: AsyncIterator,
        P: FnMut<(I::Item,)>,
        P: ?Sized,
        P::Output: IntoFuture<Output = bool>,
    {
        #[pin]
        inner: TryFoldAsync<I, (), CopyFn, AnyAsyncFn<P>>,
    }
}

impl<I, P> AnyAsync<I, P>
where
    I: AsyncIterator,
    P: FnMut<(I::Item,)>,
    P::Output: IntoFuture<Output = bool>,
{
    pub(crate) fn new(iter: I, predicate: P) -> Self {
        Self {
            inner: TryFoldAsync::new(iter, (), CopyFn::default(), AnyAsyncFn { predicate }),
        }
    }
}

impl<I, P> Clone for AnyAsync<I, P>
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

impl<I, P> Future for AnyAsync<I, P>
where
    I: AsyncIterator,
    P: FnMut<(I::Item,)> + ?Sized,
    P::Output: IntoFuture<Output = bool>,
{
    type Output = bool;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        Poll::Ready(task::ready!(self.project().inner.poll(cx)).is_break())
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iterator_ext::AsyncIteratorExt;
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
