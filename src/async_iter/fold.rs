use crate::support::{AsyncIterator, FnMut2, FusedAsyncIterator};
use core::future::Future;
use core::pin::Pin;
use core::task::{self, Context, Poll};
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    pub struct Fold<I, B, F> {
        #[pin]
        iter: I,
        acc: B,
        f: F,
    }
}

impl<I, B, F> Fold<I, B, F> {
    pub(crate) fn new(iter: I, acc: B, f: F) -> Self {
        Self { iter, acc, f }
    }
}

impl<I, B, F> Clone for Fold<I, B, F>
where
    I: Clone,
    B: Clone,
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

impl<I, B, F> Future for Fold<I, B, F>
where
    I: AsyncIterator,
    B: Copy,
    F: FnMut2<B, I::Item, Output = B>,
{
    type Output = B;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();
        let mut iter = this.iter;
        let acc = this.acc;
        let f = this.f;

        while let Some(item) = task::ready!(iter.as_mut().poll_next(cx)) {
            *acc = f.call_mut(*acc, item);
        }

        Poll::Ready(*acc)
    }
}

impl<I, B, F> FusedFuture for Fold<I, B, F>
where
    I: FusedAsyncIterator,
    B: Copy,
    F: FnMut2<B, I::Item, Output = B>,
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
        let future = stream::iter([2, 3, 5]).fold(1_u64, accumulate);

        assert_eq!(future.await, 30_u64);
    }

    #[tokio::test]
    async fn test_fold_clone() {
        let future = stream::iter([2, 3, 5]).fold(1_u64, accumulate);
        let future_2 = future.clone();

        assert_eq!(future.await, 30_u64);
        assert_eq!(future_2.await, 30_u64);
    }
}
