use crate::async_iter::try_fold::TryFold;
use crate::future::Map;
use crate::support::fns::UnwrapContinueValueFn;
use crate::support::{AsyncIterator, FusedAsyncIterator, Never};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::fns::{self, ComposeFn, ControlFlowContinueFn};
use fn_traits::FnMut;
use futures_core::FusedFuture;

type InnerFuture<I, T, G, F> = TryFold<I, T, G, ComposeFn<F, ControlFlowContinueFn<Never>>>;

pin_project_lite::pin_project! {
    pub struct Fold<I, T, G, F> {
        #[pin]
        inner: Map<InnerFuture<I, T, G, F>, UnwrapContinueValueFn>,
    }
}

impl<I, T, G, F> Fold<I, T, G, F> {
    pub(crate) fn new(iter: I, acc: T, getter: G, f: F) -> Self {
        Self {
            inner: Map::new(
                TryFold::new(iter, acc, getter, fns::compose(f, ControlFlowContinueFn::default())),
                UnwrapContinueValueFn::default(),
            ),
        }
    }
}

impl<I, T, G, F> Clone for Fold<I, T, G, F>
where
    I: Clone,
    T: Clone,
    G: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<I, T, G, F> Future for Fold<I, T, G, F>
where
    I: AsyncIterator,
    G: for<'a> FnMut<(&'a mut T,), Output = T>,
    F: FnMut<(T, I::Item), Output = T>,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<I, T, G, F> FusedFuture for Fold<I, T, G, F>
where
    I: FusedAsyncIterator,
    G: for<'a> FnMut<(&'a mut T,), Output = T>,
    F: FnMut<(T, I::Item), Output = T>,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iter_ext::AsyncIteratorExt;
    use futures_util::stream;

    fn accumulate(state: u64, item: u32) -> u64 {
        state * u64::from(item)
    }

    #[tokio::test]
    async fn test_fold() {
        let future = stream::iter([2, 3, 5]).slim_fold_by_copy(1_u64, accumulate);

        assert_eq!(future.await, 30_u64);
    }

    #[tokio::test]
    async fn test_fold_clone() {
        let future = stream::iter([2, 3, 5]).slim_fold_by_copy(1_u64, accumulate);
        let future_2 = future.clone();

        assert_eq!(future.await, 30_u64);
        assert_eq!(future_2.await, 30_u64);
    }
}
