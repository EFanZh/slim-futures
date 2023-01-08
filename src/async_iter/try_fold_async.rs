use crate::support::{AsyncIterator, FnMut2, FromResidual, Try};
use futures_core::{FusedFuture, FusedStream};
use std::future::Future;
use std::ops::ControlFlow;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    pub struct TryFoldAsync<I, B, F>
    where
        I: AsyncIterator,
        F: FnMut2<B, I::Item>,
    {
        #[pin]
        iter: I,
        acc: B,
        f: F,
        #[pin]
        fut: Option<F::Output>,
    }
}

impl<I, B, F> TryFoldAsync<I, B, F>
where
    I: AsyncIterator,
    F: FnMut2<B, I::Item>,
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

impl<I, B, F> Clone for TryFoldAsync<I, B, F>
where
    I: AsyncIterator + Clone,
    B: Clone,
    F: FnMut2<B, I::Item> + Clone,
    F::Output: Clone,
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

impl<I, B, F> Future for TryFoldAsync<I, B, F>
where
    I: AsyncIterator,
    B: Copy,
    F: FnMut2<B, I::Item>,
    F::Output: Future,
    <F::Output as Future>::Output: Try<Output = B>,
{
    type Output = <F::Output as Future>::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();
        let mut iter = this.iter;
        let acc = this.acc;
        let f = this.f;
        let mut fut = this.fut;

        loop {
            if let Some(inner_future) = fut.as_mut().as_pin_mut() {
                *acc = match futures_core::ready!(inner_future.poll(cx)).branch() {
                    ControlFlow::Continue(acc) => acc,
                    ControlFlow::Break(error) => return Poll::Ready(Self::Output::from_residual(error)),
                };

                fut.set(None);
            } else if let Some(item) = futures_core::ready!(iter.as_mut().poll_next(cx)) {
                fut.set(Some(f.call_mut(*acc, item)));
            } else {
                break;
            }
        }

        Poll::Ready(Self::Output::from_output(*acc))
    }
}

impl<I, B, F> FusedFuture for TryFoldAsync<I, B, F>
where
    I: FusedStream,
    B: Copy,
    F: FnMut2<B, I::Item>,
    F::Output: FusedFuture,
    <F::Output as Future>::Output: Try<Output = B>,
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
    use std::convert::Infallible;

    fn accumulate(state: u64, item: u32) -> Ready<Result<u64, Infallible>> {
        future::ready(Ok(state * u64::from(item)))
    }

    #[tokio::test]
    async fn test_try_fold_async() {
        let future = stream::iter([2, 3, 5]).try_fold_async(1_u64, accumulate);

        assert_eq!(future.await, Ok(30_u64));
    }

    #[tokio::test]
    async fn test_try_fold_async_error() {
        let mut counter = 0;

        let future = stream::iter([2, 3, 5]).try_fold_async(1_u64, |state, item: u32| {
            if counter < 2 {
                counter += 1;

                future::ok(state * u64::from(item))
            } else {
                future::err(7)
            }
        });

        assert_eq!(future.await, Err(7));
        assert_eq!(counter, 2);
    }

    #[tokio::test]
    async fn test_try_fold_async_clone() {
        let future = stream::iter([2, 3, 5]).try_fold_async(1_u64, accumulate);
        let future_2 = future.clone();

        assert_eq!(future.await, Ok(30_u64));
        assert_eq!(future_2.await, Ok(30_u64));
    }
}
