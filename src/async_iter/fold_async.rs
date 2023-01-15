use crate::support::{AsyncIterator, FnMut2, FusedAsyncIterator};
use core::future::{Future, IntoFuture};
use core::pin::Pin;
use core::task::{self, Context, Poll};
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    pub struct FoldAsync<I, B, F>
    where
        I: AsyncIterator,
        F: FnMut2<B, I::Item>,
        F::Output: IntoFuture,
    {
        #[pin]
        iter: I,
        acc: B,
        f: F,
        #[pin]
        fut: Option<<F::Output as IntoFuture>::IntoFuture>,
    }
}

impl<I, B, F> FoldAsync<I, B, F>
where
    I: AsyncIterator,
    F: FnMut2<B, I::Item>,
    F::Output: IntoFuture,
{
    pub(crate) fn new(iter: I, acc: B, f: F) -> Self {
        Self {
            iter,
            acc,
            f,
            fut: None,
        }
    }
}

impl<I, B, F> Clone for FoldAsync<I, B, F>
where
    I: AsyncIterator + Clone,
    B: Clone,
    F: FnMut2<B, I::Item> + Clone,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::IntoFuture: Clone,
{
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            acc: self.acc.clone(),
            f: self.f.clone(),
            fut: self.fut.clone(),
        }
    }
}

impl<I, B, F> Future for FoldAsync<I, B, F>
where
    I: AsyncIterator,
    B: Copy,
    F: FnMut2<B, I::Item>,
    F::Output: IntoFuture<Output = B>,
{
    type Output = B;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();
        let mut iter = this.iter;
        let acc = this.acc;
        let f = this.f;
        let mut fut = this.fut;

        loop {
            if let Some(inner_future) = fut.as_mut().as_pin_mut() {
                *acc = task::ready!(inner_future.poll(cx));

                fut.set(None);
            } else if let Some(item) = task::ready!(iter.as_mut().poll_next(cx)) {
                fut.set(Some(f.call_mut(*acc, item).into_future()));
            } else {
                break;
            }
        }

        Poll::Ready(*acc)
    }
}

impl<I, B, F> FusedFuture for FoldAsync<I, B, F>
where
    I: FusedAsyncIterator,
    B: Copy,
    F: FnMut2<B, I::Item>,
    F::Output: FusedFuture<Output = B>,
{
    fn is_terminated(&self) -> bool {
        if let Some(fut) = &self.fut {
            fut.is_terminated()
        } else {
            self.iter.is_terminated()
        }
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
        let future = stream::iter([2, 3, 5]).slim_fold_async(1_u64, accumulate);

        assert_eq!(future.await, 30_u64);
    }

    #[tokio::test]
    async fn test_fold_async_clone() {
        let future = stream::iter([2, 3, 5]).slim_fold_async(1_u64, accumulate);
        let future_2 = future.clone();

        assert_eq!(future.await, 30_u64);
        assert_eq!(future_2.await, 30_u64);
    }
}
