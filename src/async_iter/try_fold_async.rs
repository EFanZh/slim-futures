use crate::support::{AsyncIterator, FromResidual, FusedAsyncIterator, Try};
use core::future::{Future, IntoFuture};
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{self, Context, Poll};
use fn_traits::FnMut;
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    pub struct TryFoldAsync<I, T, F>
    where
        I: AsyncIterator,
        F: FnMut<(T, I::Item)>,
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

impl<I, T, F> TryFoldAsync<I, T, F>
where
    I: AsyncIterator,
    F: FnMut<(T, I::Item)>,
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

impl<I, T, F> Clone for TryFoldAsync<I, T, F>
where
    I: AsyncIterator + Clone,
    T: Clone,
    F: FnMut<(T, I::Item)> + Clone,
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

impl<I, T, F> Future for TryFoldAsync<I, T, F>
where
    I: AsyncIterator,
    T: Copy,
    F: FnMut<(T, I::Item)>,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::Output: Try<Output = T>,
{
    type Output = <F::Output as IntoFuture>::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();
        let mut iter = this.iter;
        let acc = this.acc;
        let f = this.f;
        let mut fut = this.fut;

        Poll::Ready(loop {
            if let Some(inner_future) = fut.as_mut().as_pin_mut() {
                *acc = match task::ready!(inner_future.poll(cx)).branch() {
                    ControlFlow::Continue(acc) => acc,
                    ControlFlow::Break(residual) => break Self::Output::from_residual(residual),
                };

                fut.set(None);
            } else if let Some(item) = task::ready!(iter.as_mut().poll_next(cx)) {
                fut.set(Some(f.call_mut((*acc, item)).into_future()));
            } else {
                break Self::Output::from_output(*acc);
            }
        })
    }
}

impl<I, T, F> FusedFuture for TryFoldAsync<I, T, F>
where
    I: FusedAsyncIterator,
    T: Copy,
    F: FnMut<(T, I::Item)>,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::Output: Try<Output = T>,
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
    use std::convert::Infallible;

    fn accumulate(state: u64, item: u32) -> Ready<Result<u64, Infallible>> {
        future::ready(Ok(state * u64::from(item)))
    }

    #[tokio::test]
    async fn test_try_fold_async() {
        let future = stream::iter([2, 3, 5]).slim_try_fold_async(1_u64, accumulate);

        assert_eq!(future.await, Ok(30_u64));
    }

    #[tokio::test]
    async fn test_try_fold_async_error() {
        let mut counter = 0;

        let future = stream::iter([2, 3, 5]).slim_try_fold_async(1_u64, |state, item: u32| {
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
        let future = stream::iter([2, 3, 5]).slim_try_fold_async(1_u64, accumulate);
        let future_2 = future.clone();

        assert_eq!(future.await, Ok(30_u64));
        assert_eq!(future_2.await, Ok(30_u64));
    }
}
