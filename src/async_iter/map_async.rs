use crate::support::{AsyncIterator, FusedAsyncIterator};
use core::future::IntoFuture;
use core::pin::Pin;
use core::task::{self, Context, Poll};
use fn_traits::FnMut;
use futures_core::{FusedFuture, Future};
use option_entry::{OptionEntryExt, OptionPinnedEntry};

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
        state: Option<<F::Output as IntoFuture>::IntoFuture>,
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
        Self { iter, state: None, f }
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
            state: self.state.clone(),
            f: self.f.clone(),
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
        let state = this.state.pinned_entry();
        let f = this.f;

        let mut fut = match state {
            OptionPinnedEntry::None(none_state) => match task::ready!(iter.as_mut().poll_next(cx)) {
                None => return Poll::Ready(None),
                Some(item) => none_state.replace_some(f.call_mut((item,)).into_future()),
            },
            OptionPinnedEntry::Some(some_state) => some_state,
        };

        let item = task::ready!(fut.get_pin_mut().poll(cx));

        fut.replace_none();

        Poll::Ready(Some(item))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut candidate = self.iter.size_hint();

        if self.state.is_some() {
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
        self.state
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
