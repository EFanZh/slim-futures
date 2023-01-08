use crate::support::{AsyncIterator, FnMut2, FromResidual, Try};
use futures_core::{FusedFuture, FusedStream};
use std::future::Future;
use std::ops::ControlFlow;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    pub struct TryFold<I, B, F> {
        #[pin]
        iter: I,
        acc: B,
        f: F,
    }
}

impl<I, B, F> TryFold<I, B, F> {
    pub(crate) fn new(iter: I, acc: B, f: F) -> Self {
        Self { iter, acc, f }
    }
}

impl<I, B, F> Clone for TryFold<I, B, F>
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

impl<I, B, F> Future for TryFold<I, B, F>
where
    I: AsyncIterator,
    B: Copy,
    F: FnMut2<B, I::Item>,
    F::Output: Try<Output = B>,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();
        let mut iter = this.iter;
        let acc = this.acc;
        let f = this.f;

        while let Some(item) = futures_core::ready!(iter.as_mut().poll_next(cx)) {
            match f.call_mut(*acc, item).branch() {
                ControlFlow::Continue(result) => *acc = result,
                ControlFlow::Break(residual) => return Poll::Ready(Self::Output::from_residual(residual)),
            }
        }

        Poll::Ready(<Self::Output>::from_output(*acc))
    }
}

impl<I, B, F> FusedFuture for TryFold<I, B, F>
where
    I: FusedStream,
    B: Copy,
    F: FnMut2<B, I::Item>,
    F::Output: Try<Output = B>,
{
    fn is_terminated(&self) -> bool {
        self.iter.is_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iter_ext::AsyncIteratorExt;
    use futures_util::stream;
    use std::convert::Infallible;

    #[allow(clippy::unnecessary_wraps)]
    fn accumulate_1(state: u64, item: u32) -> Result<u64, Infallible> {
        Ok(state * u64::from(item))
    }

    #[tokio::test]
    async fn test_try_fold() {
        let future = stream::iter([2, 3, 5]).try_fold(1_u64, accumulate_1);

        assert_eq!(future.await, Ok(30_u64));
    }

    #[tokio::test]
    async fn test_try_fold_error() {
        let mut counter = 0;

        let future = stream::iter([2, 3, 5]).try_fold(1_u64, |state, item: u32| {
            if counter < 2 {
                counter += 1;

                Ok(state * u64::from(item))
            } else {
                Err(7)
            }
        });

        assert_eq!(future.await, Err(7));
        assert_eq!(counter, 2);
    }

    #[tokio::test]
    async fn test_try_fold_clone() {
        let future = stream::iter([2, 3, 5]).try_fold(1_u64, accumulate_1);
        let future_2 = future.clone();

        assert_eq!(future.await, Ok(30_u64));
        assert_eq!(future_2.await, Ok(30_u64));
    }
}
