use crate::async_iter::try_fold_async::TryFoldAsync;
use crate::future::Map;
use crate::support::fns::UnwrapContinueValueFn;
use crate::support::{AsyncIterator, Never};
use core::future::{Future, IntoFuture};
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::fns::ControlFlowContinueFn;
use fn_traits::FnMut;

#[derive(Clone)]
struct FoldAsyncFn<F> {
    f: F,
}

impl<T, U, F> FnMut<(T, U)> for FoldAsyncFn<F>
where
    F: FnMut<(T, U)>,
    F::Output: IntoFuture,
{
    type Output = Map<<F::Output as IntoFuture>::IntoFuture, ControlFlowContinueFn<Never>>;

    fn call_mut(&mut self, args: (T, U)) -> Self::Output {
        Map::new(self.f.call_mut(args).into_future(), ControlFlowContinueFn::default())
    }
}

pin_project_lite::pin_project! {
    pub struct FoldAsync<I, T, G, F>
    where
        I: AsyncIterator,
        F: FnMut<(T, I::Item)>,
        F::Output: IntoFuture,
    {
        #[pin]
        inner: Map<TryFoldAsync<I, T, G, FoldAsyncFn<F>>, UnwrapContinueValueFn>
    }
}

impl<I, T, G, F> FoldAsync<I, T, G, F>
where
    I: AsyncIterator,
    F: FnMut<(T, I::Item)>,
    F::Output: IntoFuture<Output = T>,
{
    pub(crate) fn new(iter: I, acc: T, getter: G, f: F) -> Self {
        Self {
            inner: Map::new(
                TryFoldAsync::new(iter, acc, getter, FoldAsyncFn { f }),
                UnwrapContinueValueFn::default(),
            ),
        }
    }
}

impl<I, T, G, F> Clone for FoldAsync<I, T, G, F>
where
    I: AsyncIterator + Clone,
    T: Clone,
    G: Clone,
    F: FnMut<(T, I::Item)> + Clone,
    F::Output: IntoFuture<Output = T>,
    <F::Output as IntoFuture>::IntoFuture: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<I, T, G, F> Future for FoldAsync<I, T, G, F>
where
    I: AsyncIterator,
    G: for<'a> FnMut<(&'a mut T,), Output = T>,
    F: FnMut<(T, I::Item)>,
    F::Output: IntoFuture<Output = T>,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iter_ext::AsyncIteratorExt;
    use futures_util::future::{self, Ready};
    use futures_util::stream;

    fn accumulate(state: u64, item: u32) -> Ready<u64> {
        future::ready(state * u64::from(item))
    }

    #[tokio::test]
    async fn test_fold_async() {
        let future = stream::iter([2, 3, 5]).slim_fold_async_by_copy(1_u64, accumulate);

        assert_eq!(future.await, 30_u64);
    }

    #[tokio::test]
    async fn test_fold_async_clone() {
        let future = stream::iter([2, 3, 5]).slim_fold_async_by_copy(1_u64, accumulate);
        let future_2 = future.clone();

        assert_eq!(future.await, 30_u64);
        assert_eq!(future_2.await, 30_u64);
    }
}
