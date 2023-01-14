use crate::support::{AsyncIterator, FnMut1, FusedAsyncIterator};
use core::pin::Pin;
use core::task::{self, Context, Poll};

pin_project_lite::pin_project! {
    pub struct Filter<I, P> {
        #[pin]
        iter: I,
        predicate: P,
    }
}

impl<I, P> Filter<I, P> {
    pub(crate) fn new(iter: I, predicate: P) -> Self {
        Self { iter, predicate }
    }
}

impl<I, P> Clone for Filter<I, P>
where
    I: Clone,
    P: Clone,
{
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            predicate: self.predicate.clone(),
        }
    }
}

impl<I, P> AsyncIterator for Filter<I, P>
where
    I: AsyncIterator,
    P: for<'a> FnMut1<&'a I::Item, Output = bool>,
{
    type Item = I::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let mut iter = this.iter;
        let predicate = this.predicate;

        while let Some(item) = task::ready!(iter.as_mut().poll_next(cx)) {
            if predicate.call_mut(&item) {
                return Poll::Ready(Some(item));
            }
        }

        Poll::Ready(None)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.iter.size_hint().1)
    }
}

impl<I, P> FusedAsyncIterator for Filter<I, P>
where
    I: FusedAsyncIterator,
    P: for<'a> FnMut1<&'a I::Item, Output = bool>,
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

    #[tokio::test]
    async fn test_filter() {
        let iter = AsyncIteratorExt::filter(stream::iter(0..10), |&x| x % 2 == 0);

        assert_eq!(iter.collect::<Vec<_>>().await, [0, 2, 4, 6, 8]);
    }

    #[tokio::test]
    async fn test_filter_clone() {
        let iter = AsyncIteratorExt::filter(stream::iter(0..10), |&x| x % 2 == 0);
        let iter_2 = iter.clone();

        assert_eq!(iter.collect::<Vec<_>>().await, [0, 2, 4, 6, 8]);
        assert_eq!(iter_2.collect::<Vec<_>>().await, [0, 2, 4, 6, 8]);
    }
}
