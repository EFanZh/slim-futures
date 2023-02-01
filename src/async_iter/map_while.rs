use crate::support::AsyncIterator;
use core::pin::Pin;
use core::task::{self, Context, Poll};
use fn_traits::FnMut;

pin_project_lite::pin_project! {
    pub struct MapWhile<I, F>
    where
        F: ?Sized,
    {
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
    F: FnMut<(I::Item,), Output = Option<T>> + ?Sized,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();

        Poll::Ready(task::ready!(this.iter.poll_next(cx)).and_then(|item| this.f.call_mut((item,))))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.iter.size_hint().1)
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iterator_ext::AsyncIteratorExt;
    use futures_util::{stream, StreamExt};
    use std::vec::Vec;

    fn map_while_fn(x: u32) -> Option<u32> {
        (x < 5).then_some(x * 10)
    }

    #[tokio::test]
    async fn test_map_while() {
        let iter = stream::iter(0..10).slim_map_while(map_while_fn);

        assert_eq!(iter.collect::<Vec<_>>().await, [0, 10, 20, 30, 40]);
    }

    #[tokio::test]
    async fn test_map_while_clone() {
        let iter = stream::iter(0..10).slim_map_while(map_while_fn);
        let iter_2 = iter.clone();

        assert_eq!(iter.collect::<Vec<_>>().await, [0, 10, 20, 30, 40]);
        assert_eq!(iter_2.collect::<Vec<_>>().await, [0, 10, 20, 30, 40],);
    }
}
