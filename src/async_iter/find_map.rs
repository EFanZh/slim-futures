use crate::async_iter::filter_map::FilterMap;
use crate::support::AsyncIterator;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::FnMut;

pin_project_lite::pin_project! {
    pub struct FindMap<I, F>
    where
        F: ?Sized,
    {
        #[pin]
        inner: FilterMap<I, F>
    }
}

impl<I, F> FindMap<I, F> {
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self {
            inner: FilterMap::new(iter, f),
        }
    }
}

impl<I, F> Clone for FindMap<I, F>
where
    I: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<I, F, T> Future for FindMap<I, F>
where
    I: AsyncIterator,
    F: FnMut<(I::Item,), Output = Option<T>> + ?Sized,
{
    type Output = Option<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll_next(cx)
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iterator_ext::AsyncIteratorExt;
    use futures_util::stream;

    fn find_3_then_mul_10(x: u32) -> Option<u64> {
        (x == 3).then_some(u64::from(x) * 10)
    }

    fn find_7_then_mul_10(x: u32) -> Option<u64> {
        (x == 7).then_some(u64::from(x) * 10)
    }

    #[tokio::test]
    async fn test_find_map() {
        let future = stream::iter([2, 3, 5]).slim_find_map(find_3_then_mul_10);

        assert_eq!(future.await, Some(30));
    }

    #[tokio::test]
    async fn test_find_map_fail() {
        let future = stream::iter([2, 3, 5]).slim_find_map(find_7_then_mul_10);

        assert!(future.await.is_none());
    }

    #[tokio::test]
    async fn test_find_map_clone() {
        let future = stream::iter([2, 3, 5]).slim_find_map(find_3_then_mul_10);
        let future_2 = future.clone();

        assert_eq!(future.await, Some(30));
        assert_eq!(future_2.await, Some(30));
    }
}
