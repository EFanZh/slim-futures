use crate::async_iter::filter_map::FilterMap;
use crate::support::{AsyncIterator, FnMut1, FusedAsyncIterator};
use core::pin::Pin;
use core::task::{Context, Poll};

#[derive(Clone)]
struct FilterFn<P> {
    predicate: P,
}

impl<T, P> FnMut1<T> for FilterFn<P>
where
    P: for<'a> FnMut1<&'a T, Output = bool>,
{
    type Output = Option<T>;

    fn call_mut(&mut self, arg: T) -> Self::Output {
        self.predicate.call_mut(&arg).then_some(arg)
    }
}

pin_project_lite::pin_project! {
    pub struct Filter<I, P> {
        #[pin]
        inner: FilterMap<I, FilterFn<P>>,
    }
}

impl<I, P> Filter<I, P> {
    pub(crate) fn new(iter: I, predicate: P) -> Self {
        Self {
            inner: FilterMap::new(iter, FilterFn { predicate }),
        }
    }
}

impl<I, P> Clone for Filter<I, P>
where
    I: Clone,
    P: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
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
        self.project().inner.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<I, P> FusedAsyncIterator for Filter<I, P>
where
    I: FusedAsyncIterator,
    P: for<'a> FnMut1<&'a I::Item, Output = bool>,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iter_ext::AsyncIteratorExt;
    use futures_util::{stream, StreamExt};
    use std::vec::Vec;

    #[tokio::test]
    async fn test_filter() {
        let iter = stream::iter(0..10).slim_filter(|&x| x % 2 == 0);

        assert_eq!(iter.collect::<Vec<_>>().await, [0, 2, 4, 6, 8]);
    }

    #[tokio::test]
    async fn test_filter_clone() {
        let iter = stream::iter(0..10).slim_filter(|&x| x % 2 == 0);
        let iter_2 = iter.clone();

        assert_eq!(iter.collect::<Vec<_>>().await, [0, 2, 4, 6, 8]);
        assert_eq!(iter_2.collect::<Vec<_>>().await, [0, 2, 4, 6, 8]);
    }
}
