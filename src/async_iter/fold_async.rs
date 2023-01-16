use crate::support::{AsyncIterator, FnMut2, FusedAsyncIterator};
use core::future::{Future, IntoFuture};
use core::pin::Pin;
use core::task::{self, Context, Poll};
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    pub struct FoldAsync<I, T, F>
    where
        I: AsyncIterator,
        F: FnMut2<T, I::Item>,
        F::Output: IntoFuture,
    {
        #[pin]
        iter: I,
        acc: T,
        f: F,
        #[pin]
        fut: Option<<F::Output as IntoFuture>::IntoFuture>,
    }
}

impl<I, T, F> FoldAsync<I, T, F>
where
    I: AsyncIterator,
    F: FnMut2<T, I::Item>,
    F::Output: IntoFuture,
{
    pub(crate) fn new(iter: I, acc: T, f: F) -> Self {
        Self {
            iter,
            acc,
            f,
            fut: None,
        }
    }
}

impl<I, T, F> Clone for FoldAsync<I, T, F>
where
    I: AsyncIterator + Clone,
    T: Clone,
    F: FnMut2<T, I::Item> + Clone,
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

impl<I, T, F> Future for FoldAsync<I, T, F>
where
    I: AsyncIterator,
    T: Copy,
    F: FnMut2<T, I::Item>,
    F::Output: IntoFuture<Output = T>,
{
    type Output = T;

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

impl<I, T, F> FusedFuture for FoldAsync<I, T, F>
where
    I: FusedAsyncIterator,
    T: Copy,
    F: FnMut2<T, I::Item>,
    F::Output: IntoFuture<Output = T>,
    <F::Output as IntoFuture>::IntoFuture: FusedFuture,
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
