use crate::support::{AsyncIterator, FusedAsyncIterator, OptionExt};
use core::future::IntoFuture;
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{self, Context, Poll};
use fn_traits::FnMut;
use futures_core::{FusedFuture, Future};

pin_project_lite::pin_project! {
    pub struct MapAsync<I, F>
    where
        I: AsyncIterator,
        F: FnMut<(I::Item,)>,
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

impl<I, F> MapAsync<I, F>
where
    I: AsyncIterator,
    F: FnMut<(I::Item,)>,
    F::Output: IntoFuture,
{
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self { iter, fut: None, f }
    }
}

impl<I, F> Clone for MapAsync<I, F>
where
    I: AsyncIterator + Clone,
    F: FnMut<(I::Item,)> + Clone,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::IntoFuture: Clone,
{
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            f: self.f.clone(),
            fut: self.fut.clone(),
        }
    }
}

impl<I, F> AsyncIterator for MapAsync<I, F>
where
    I: AsyncIterator,
    F: FnMut<(I::Item,)> + ?Sized,
    F::Output: IntoFuture,
{
    type Item = <F::Output as IntoFuture>::Output;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let mut iter = this.iter;
        let mut fut_slot = this.fut;
        let f = this.f;

        match fut_slot.as_mut().get_or_try_insert_with_pinned(|| {
            ControlFlow::Break(match iter.as_mut().poll_next(cx) {
                Poll::Ready(item) => match item {
                    None => Poll::Ready(None),
                    Some(item) => return ControlFlow::Continue(f.call_mut((item,)).into_future()),
                },
                Poll::Pending => Poll::Pending,
            })
        }) {
            ControlFlow::Continue(fut) => {
                let item = task::ready!(fut.poll(cx));

                fut_slot.set(None);

                Poll::Ready(Some(item))
            }
            ControlFlow::Break(result) => result,
        }
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

impl<I, F> FusedAsyncIterator for MapAsync<I, F>
where
    I: FusedAsyncIterator,
    F: FnMut<(I::Item,)> + ?Sized,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::IntoFuture: FusedFuture,
{
    fn is_terminated(&self) -> bool {
        self.fut
            .as_ref()
            .map_or_else(|| self.iter.is_terminated(), FusedFuture::is_terminated)
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iterator_ext::AsyncIteratorExt;
    use futures_util::future::{self, Ready};
    use futures_util::{stream, StreamExt};
    use std::vec::Vec;

    fn map_fn(x: u32) -> Ready<u64> {
        future::ready(u64::from(x) * 10)
    }

    #[tokio::test]
    async fn test_map_async() {
        let iter = stream::iter(0..10).slim_map_async(map_fn);

        assert_eq!(iter.collect::<Vec<_>>().await, [0, 10, 20, 30, 40, 50, 60, 70, 80, 90]);
    }

    #[tokio::test]
    async fn test_map_async_clone() {
        let iter = stream::iter(0..10).slim_map_async(map_fn);
        let iter_2 = iter.clone();

        assert_eq!(iter.collect::<Vec<_>>().await, [0, 10, 20, 30, 40, 50, 60, 70, 80, 90]);

        assert_eq!(
            iter_2.collect::<Vec<_>>().await,
            [0, 10, 20, 30, 40, 50, 60, 70, 80, 90],
        );
    }
}
