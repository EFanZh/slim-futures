use crate::support::{AsyncIterator, FromResidual, FusedAsyncIterator, Try};
use core::future::Future;
use core::future::IntoFuture;
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{self, Context, Poll};
use fn_traits::FnMut;
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    pub struct AndThenAsync<I, F>
    where
        I: AsyncIterator,
        I::Item: Try,
        F: FnMut<(<I::Item as Try>::Output,)>,
        F: ?Sized,
        F::Output: IntoFuture,
    {
        #[pin]
        iter: I,
        #[pin]
        fut: Option<<F::Output as IntoFuture>::IntoFuture>,
        f: F,
    }
}

impl<I, F> AndThenAsync<I, F>
where
    I: AsyncIterator,
    I::Item: Try,
    F: FnMut<(<I::Item as Try>::Output,)>,
    F::Output: IntoFuture,
{
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self { iter, fut: None, f }
    }
}

impl<I, F> Clone for AndThenAsync<I, F>
where
    I: AsyncIterator + Clone,
    I::Item: Try,
    F: FnMut<(<I::Item as Try>::Output,)> + Clone,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::IntoFuture: Clone,
{
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            fut: self.fut.clone(),
            f: self.f.clone(),
        }
    }
}

impl<I, F> AsyncIterator for AndThenAsync<I, F>
where
    I: AsyncIterator,
    I::Item: Try,
    F: FnMut<(<I::Item as Try>::Output,)> + ?Sized,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::Output: FromResidual<<I::Item as Try>::Residual>,
{
    type Item = <F::Output as IntoFuture>::Output;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let mut iter = this.iter;
        let f = this.f;
        let mut fut_slot = this.fut;

        Poll::Ready(loop {
            match fut_slot.as_mut().as_pin_mut() {
                None => {}
                Some(fut) => {
                    let item = task::ready!(fut.poll(cx));

                    fut_slot.set(None);

                    break Some(item);
                }
            }

            match task::ready!(iter.as_mut().poll_next(cx)) {
                None => break None,
                Some(item) => match item.branch() {
                    ControlFlow::Continue(output) => fut_slot.set(Some(f.call_mut((output,)).into_future())),
                    ControlFlow::Break(residual) => break Some(Self::Item::from_residual(residual)),
                },
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut candidate = self.iter.size_hint();

        if self.fut.is_some() {
            candidate.0 = candidate.0.saturating_add(1);
            candidate.1 = candidate.1.and_then(|high| high.checked_add(1));
        }

        candidate
    }
}

impl<I, F> FusedAsyncIterator for AndThenAsync<I, F>
where
    I: FusedAsyncIterator,
    I::Item: Try,
    F: FnMut<(<I::Item as Try>::Output,)> + ?Sized,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::Output: FromResidual<<I::Item as Try>::Residual>,
    <F::Output as IntoFuture>::IntoFuture: FusedFuture,
{
    fn is_terminated(&self) -> bool {
        match &self.fut {
            None => self.iter.is_terminated(),
            Some(fut) => fut.is_terminated(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iter_ext::AsyncIteratorExt;
    use futures_util::future::Ready;
    use futures_util::{future, stream, StreamExt};
    use std::vec::Vec;

    fn and_then_async_fn(x: u32) -> Ready<Result<u64, u64>> {
        if x < 12 {
            future::ok(u64::from(x * 100))
        } else {
            future::err(u64::from(x * 1000))
        }
    }

    #[tokio::test]
    async fn test_and_then_async() {
        let iter =
            stream::iter([Ok(2), Ok(3), Err(5_u32), Err(7), Ok(11), Ok(13)]).slim_and_then_async(and_then_async_fn);

        assert_eq!(
            iter.collect::<Vec<_>>().await,
            [Ok(200), Ok(300), Err(5), Err(7), Ok(1100), Err(13000)],
        );
    }

    #[tokio::test]
    async fn test_and_then_async_clone() {
        let iter =
            stream::iter([Ok(2), Ok(3), Err(5_u32), Err(7), Ok(11), Ok(13)]).slim_and_then_async(and_then_async_fn);
        let iter_2 = iter.clone();

        assert_eq!(
            iter.collect::<Vec<_>>().await,
            [Ok(200), Ok(300), Err(5), Err(7), Ok(1100), Err(13000)],
        );

        assert_eq!(
            iter_2.collect::<Vec<_>>().await,
            [Ok(200), Ok(300), Err(5), Err(7), Ok(1100), Err(13000)],
        );
    }
}
