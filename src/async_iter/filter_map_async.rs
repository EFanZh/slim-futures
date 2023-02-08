use crate::support::{AsyncIterator, FusedAsyncIterator};
use core::future::IntoFuture;
use core::pin::Pin;
use core::task::{self, Context, Poll};
use fn_traits::FnMut;
use futures_core::{FusedFuture, Future};
use option_entry::{OptionEntryExt, OptionPinnedEntry};

pin_project_lite::pin_project! {
    pub struct FilterMapAsync<I, F>
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

impl<I, F> FilterMapAsync<I, F>
where
    I: AsyncIterator,
    F: FnMut<(I::Item,)>,
    F::Output: IntoFuture,
{
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self { iter, state: None, f }
    }
}

impl<I, F> Clone for FilterMapAsync<I, F>
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

impl<I, F, T> AsyncIterator for FilterMapAsync<I, F>
where
    I: AsyncIterator,
    F: FnMut<(I::Item,)> + ?Sized,
    F::Output: IntoFuture<Output = Option<T>>,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let mut iter = this.iter;
        let mut state = this.state.pinned_entry();
        let f = this.f;

        loop {
            let mut fut = match state {
                OptionPinnedEntry::None(none_state) => match task::ready!(iter.as_mut().poll_next(cx)) {
                    None => break Poll::Ready(None),
                    Some(item) => none_state.replace_some(f.call_mut((item,)).into_future()),
                },
                OptionPinnedEntry::Some(some_state) => some_state,
            };

            let item = task::ready!(fut.get_pin_mut().poll(cx));

            state = OptionPinnedEntry::None(fut.replace_none());

            if let Some(item) = item {
                break Poll::Ready(Some(item));
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut candidate = (0, self.iter.size_hint().1);

        if self.state.is_some() {
            candidate.1 = candidate.1.and_then(|high| high.checked_add(1));
        }

        candidate
    }
}

impl<I, F, T> FusedAsyncIterator for FilterMapAsync<I, F>
where
    I: FusedAsyncIterator,
    F: FnMut<(I::Item,)> + ?Sized,
    F::Output: IntoFuture<Output = Option<T>>,
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

    fn filter_map_fn(x: u32) -> Ready<Option<u32>> {
        future::ready((x % 2 == 0).then_some(x * 10))
    }

    #[tokio::test]
    async fn test_filter_map_async() {
        let iter = stream::iter(0..10).slim_filter_map_async(filter_map_fn);

        assert_eq!(iter.collect::<Vec<_>>().await, [0, 20, 40, 60, 80]);
    }

    #[tokio::test]
    async fn test_filter_map_async_clone() {
        let iter = stream::iter(0..10).slim_filter_map_async(filter_map_fn);
        let iter_2 = iter.clone();

        assert_eq!(iter.collect::<Vec<_>>().await, [0, 20, 40, 60, 80]);
        assert_eq!(iter_2.collect::<Vec<_>>().await, [0, 20, 40, 60, 80]);
    }
}
