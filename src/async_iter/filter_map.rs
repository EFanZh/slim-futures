use crate::support::{AsyncIterator, FnMut1, FusedAsyncIterator};
use core::pin::Pin;
use core::task::{self, Context, Poll};

pin_project_lite::pin_project! {
    pub struct FilterMap<I, F> {
        #[pin]
        iter: I,
        f: F,
    }
}

impl<I, F> FilterMap<I, F> {
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self { iter, f }
    }
}

impl<I, F> Clone for FilterMap<I, F>
where
    I: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            f: self.f.clone(),
        }
    }
}

impl<I, F, B> AsyncIterator for FilterMap<I, F>
where
    I: AsyncIterator,
    F: FnMut1<I::Item, Output = Option<B>>,
{
    type Item = B;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let mut iter = this.iter;
        let f = this.f;

        while let Some(item) = task::ready!(iter.as_mut().poll_next(cx)) {
            let item = f.call_mut(item);

            if item.is_some() {
                return Poll::Ready(item);
            }
        }

        Poll::Ready(None)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.iter.size_hint().1)
    }
}

impl<I, F, B> FusedAsyncIterator for FilterMap<I, F>
where
    I: FusedAsyncIterator,
    F: FnMut1<I::Item, Output = Option<B>>,
{
    fn is_terminated(&self) -> bool {
        self.iter.is_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iter_ext::AsyncIteratorExt;
    use futures_util::{stream, StreamExt};
    use std::vec::Vec;

    fn filter_map_fn(x: u32) -> Option<u32> {
        (x % 2 == 0).then_some(x * 10)
    }

    #[tokio::test]
    async fn test_filter_map() {
        let iter = stream::iter(0..10).slim_filter_map(filter_map_fn);

        assert_eq!(iter.collect::<Vec<_>>().await, [0, 20, 40, 60, 80]);
    }

    #[tokio::test]
    async fn test_filter_map_clone() {
        let iter = stream::iter(0..10).slim_filter_map(filter_map_fn);
        let iter_2 = iter.clone();

        assert_eq!(iter.collect::<Vec<_>>().await, [0, 20, 40, 60, 80]);
        assert_eq!(iter_2.collect::<Vec<_>>().await, [0, 20, 40, 60, 80]);
    }
}