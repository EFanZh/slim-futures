use crate::support::{AsyncIterator, FnMut1, FusedAsyncIterator};
use core::pin::Pin;
use core::task::{self, Context, Poll};

pin_project_lite::pin_project! {
    pub struct MapWhile<I, F> {
        #[pin]
        iter: I,
        f: F,
    }
}

impl<I, F> MapWhile<I, F> {
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self { iter, f }
    }
}

impl<I, F> Clone for MapWhile<I, F>
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

impl<I, F, T> AsyncIterator for MapWhile<I, F>
where
    I: AsyncIterator,
    F: FnMut1<I::Item, Output = Option<T>>,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();

        Poll::Ready(task::ready!(this.iter.poll_next(cx)).and_then(|item| this.f.call_mut(item)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.iter.size_hint().1)
    }
}

impl<I, F, T> FusedAsyncIterator for MapWhile<I, F>
where
    I: FusedAsyncIterator,
    F: FnMut1<I::Item, Output = Option<T>>,
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

    fn map_fn(x: u32) -> u64 {
        u64::from(x) * 10
    }

    #[tokio::test]
    async fn test_map() {
        let iter = stream::iter(0..10).slim_map(map_fn);

        assert_eq!(iter.collect::<Vec<_>>().await, [0, 10, 20, 30, 40, 50, 60, 70, 80, 90]);
    }

    #[tokio::test]
    async fn test_map_clone() {
        let iter = stream::iter(0..10).slim_map(map_fn);
        let iter_2 = iter.clone();

        assert_eq!(iter.collect::<Vec<_>>().await, [0, 10, 20, 30, 40, 50, 60, 70, 80, 90]);

        assert_eq!(
            iter_2.collect::<Vec<_>>().await,
            [0, 10, 20, 30, 40, 50, 60, 70, 80, 90],
        );
    }
}