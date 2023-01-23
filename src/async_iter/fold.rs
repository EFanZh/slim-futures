use crate::support::{AsyncIterator, FusedAsyncIterator};
use core::future::Future;
use core::pin::Pin;
use core::task::{self, Context, Poll};
use fn_traits::FnMut;
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    pub struct Fold<I, T, G, F>
    where
        F: ?Sized,
    {
        #[pin]
        iter: I,
        acc: T,
        getter: G,
        f: F,
    }
}

impl<I, T, G, F> Fold<I, T, G, F> {
    pub(crate) fn new(iter: I, acc: T, f: F) -> Self
    where
        G: Default,
    {
        Self::with_getter(iter, acc, G::default(), f)
    }

    pub(crate) fn with_getter(iter: I, acc: T, getter: G, f: F) -> Self {
        Self { iter, acc, getter, f }
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
            iter: self.iter.clone(),
            acc: self.acc.clone(),
            getter: self.getter.clone(),
            f: self.f.clone(),
        }
    }
}

impl<I, T, G, F> Future for Fold<I, T, G, F>
where
    I: AsyncIterator,
    G: for<'a> FnMut<(&'a mut T,), Output = T>,
    F: FnMut<(T, I::Item), Output = T> + ?Sized,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();
        let mut iter = this.iter;
        let acc = this.acc;
        let getter = this.getter;
        let f = this.f;

        while let Some(item) = task::ready!(iter.as_mut().poll_next(cx)) {
            *acc = f.call_mut((getter.call_mut((acc,)), item));
        }

        Poll::Ready(getter.call_mut((acc,)))
    }
}

impl<I, T, G, F> FusedFuture for Fold<I, T, G, F>
where
    I: FusedAsyncIterator,
    G: for<'a> FnMut<(&'a mut T,), Output = T>,
    F: FnMut<(T, I::Item), Output = T> + ?Sized,
{
    fn is_terminated(&self) -> bool {
        self.iter.is_terminated()
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
