use crate::support::{AsyncIterator, FusedAsyncIterator};
use core::future::Future;
use core::pin::Pin;
use core::task::{self, Context, Poll};
use fn_traits::FnMut;
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    pub struct Fold<I, T, F> {
        #[pin]
        iter: I,
        acc: T,
        f: F,
    }
}

impl<I, T, F> Fold<I, T, F> {
    pub(crate) fn new(iter: I, acc: T, f: F) -> Self {
        Self { iter, acc, f }
    }
}

impl<I, T, F> Clone for Fold<I, T, F>
where
    I: Clone,
    T: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            acc: self.acc.clone(),
            f: self.f.clone(),
        }
    }
}

impl<I, T, F> Future for Fold<I, T, F>
where
    I: AsyncIterator,
    T: Copy,
    F: FnMut<(T, I::Item), Output = T>,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();
        let mut iter = this.iter;
        let acc = this.acc;
        let f = this.f;

        while let Some(item) = task::ready!(iter.as_mut().poll_next(cx)) {
            *acc = f.call_mut((*acc, item));
        }

        Poll::Ready(*acc)
    }
}

impl<I, T, F> FusedFuture for Fold<I, T, F>
where
    I: FusedAsyncIterator,
    T: Copy,
    F: FnMut<(T, I::Item), Output = T>,
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
        let future = stream::iter([2, 3, 5]).slim_fold(1_u64, accumulate);

        assert_eq!(future.await, 30_u64);
    }

    #[tokio::test]
    async fn test_fold_clone() {
        let future = stream::iter([2, 3, 5]).slim_fold(1_u64, accumulate);
        let future_2 = future.clone();

        assert_eq!(future.await, 30_u64);
        assert_eq!(future_2.await, 30_u64);
    }
}
