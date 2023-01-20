use crate::async_iter::try_fold_async::TryFoldAsync;
use crate::future::Map;
use crate::support::fns::ControlFlowBreakValueFn;
use crate::support::{AsyncIterator, FusedAsyncIterator, OptionFuture};
use core::future::{Future, IntoFuture};
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::FnMut;
use futures_core::FusedFuture;

#[derive(Clone)]
struct BreakIfSome;

impl<T> FnMut<(Option<T>,)> for BreakIfSome {
    type Output = ControlFlow<T>;

    fn call_mut(&mut self, args: (Option<T>,)) -> Self::Output {
        match args.0 {
            None => ControlFlow::Continue(()),
            Some(value) => ControlFlow::Break(value),
        }
    }
}

#[derive(Clone)]
struct FindMapAsyncFn<F> {
    f: F,
}

impl<T, F> FnMut<((), T)> for FindMapAsyncFn<F>
where
    F: FnMut<(T,)>,
    F::Output: IntoFuture,
{
    type Output = Map<<F::Output as IntoFuture>::IntoFuture, BreakIfSome>;

    fn call_mut(&mut self, args: ((), T)) -> Self::Output {
        Map::new(self.f.call_mut((args.1,)).into_future(), BreakIfSome)
    }
}

pin_project_lite::pin_project! {
    pub struct FindMapAsync<I, F>
    where
        I: AsyncIterator,
        F: FnMut<(I::Item,)>,
        F::Output: IntoFuture,
        <F::Output as IntoFuture>::IntoFuture: OptionFuture,
    {
        #[pin]
        inner: Map<TryFoldAsync<I, (), FindMapAsyncFn<F>>, ControlFlowBreakValueFn>
    }
}

impl<I, F> FindMapAsync<I, F>
where
    I: AsyncIterator,
    F: FnMut<(I::Item,)>,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::IntoFuture: OptionFuture,
{
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self {
            inner: Map::new(
                TryFoldAsync::new(iter, (), FindMapAsyncFn { f }),
                ControlFlowBreakValueFn::default(),
            ),
        }
    }
}

impl<I, F> Clone for FindMapAsync<I, F>
where
    I: AsyncIterator + Clone,
    F: FnMut<(I::Item,)> + Clone,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::IntoFuture: OptionFuture + Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<I, F, T> Future for FindMapAsync<I, F>
where
    I: AsyncIterator,
    F: FnMut<(I::Item,)>,
    F::Output: IntoFuture<Output = Option<T>>,
{
    type Output = Option<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<I, F, T> FusedFuture for FindMapAsync<I, F>
where
    I: FusedAsyncIterator,
    F: FnMut<(I::Item,)>,
    F::Output: FusedFuture<Output = Option<T>>,
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

    fn find_3_then_mul_10(x: u32) -> Ready<Option<u64>> {
        future::ready((x == 3).then_some(u64::from(x) * 10))
    }

    fn find_7_then_mul_10(x: u32) -> Ready<Option<u64>> {
        future::ready((x == 7).then_some(u64::from(x) * 10))
    }

    #[tokio::test]
    async fn test_find_map_async() {
        let future = stream::iter([2, 3, 5]).slim_find_map_async(find_3_then_mul_10);

        assert_eq!(future.await, Some(30));
    }

    #[tokio::test]
    async fn test_find_map_async_fail() {
        let future = stream::iter([2, 3, 5]).slim_find_map_async(find_7_then_mul_10);

        assert!(future.await.is_none());
    }

    #[tokio::test]
    async fn test_find_map_async_clone() {
        let future = stream::iter([2, 3, 5]).slim_find_map_async(find_3_then_mul_10);
        let future_2 = future.clone();

        assert_eq!(future.await, Some(30));
        assert_eq!(future_2.await, Some(30));
    }
}
