use crate::support::{AsyncIterator, FromResidual, Try};
use core::future::Future;
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{self, Context, Poll};
use fn_traits::FnMut;

pin_project_lite::pin_project! {
    pub struct TryFold<I, T, G, F>
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

impl<I, T, G, F> TryFold<I, T, G, F> {
    pub(crate) fn new(iter: I, acc: T, getter: G, f: F) -> Self {
        Self { iter, acc, getter, f }
    }
}

impl<I, T, G, F> Clone for TryFold<I, T, G, F>
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

impl<I, T, G, F> Future for TryFold<I, T, G, F>
where
    I: AsyncIterator,
    G: for<'a> FnMut<(&'a mut T,), Output = T>,
    F: FnMut<(T, I::Item)> + ?Sized,
    F::Output: Try<Output = T>,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();
        let mut iter = this.iter;
        let acc = this.acc;
        let getter = this.getter;
        let f = this.f;

        Poll::Ready(loop {
            match task::ready!(iter.as_mut().poll_next(cx)) {
                None => break <Self::Output>::from_output(getter.call_mut((acc,))),
                Some(item) => match f.call_mut((getter.call_mut((acc,)), item)).branch() {
                    ControlFlow::Continue(result) => *acc = result,
                    ControlFlow::Break(residual) => break Self::Output::from_residual(residual),
                },
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iterator_ext::AsyncIteratorExt;
    use futures_util::stream;
    use std::convert::Infallible;

    #[allow(clippy::unnecessary_wraps)]
    fn accumulate_1(state: u64, item: u32) -> Result<u64, Infallible> {
        Ok(state * u64::from(item))
    }

    #[tokio::test]
    async fn test_try_fold() {
        let future = stream::iter([2, 3, 5]).slim_try_fold_by_copy(1_u64, accumulate_1);

        assert_eq!(future.await, Ok(30_u64));
    }

    #[tokio::test]
    async fn test_try_fold_async_with_option() {
        let future =
            stream::iter([2, 3, 5]).slim_try_fold_by_copy(1_u64, |state, item: u32| Some(state * u64::from(item)));

        assert_eq!(future.await, Some(30_u64));
    }

    #[tokio::test]
    async fn test_try_fold_error() {
        let mut counter = 0;

        let future = stream::iter([2, 3, 5]).slim_try_fold_by_copy(1_u64, |state, item: u32| {
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
        let future = stream::iter([2, 3, 5]).slim_try_fold_by_take(1_u64, accumulate_1);
        let future_2 = future.clone();

        assert_eq!(future.await, Ok(30_u64));
        assert_eq!(future_2.await, Ok(30_u64));
    }
}
