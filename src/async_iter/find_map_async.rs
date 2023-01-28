use crate::async_iter::filter_map_async::FilterMapAsync;
use crate::support::{AsyncIterator, OptionFuture};
use core::future::{Future, IntoFuture};
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::FnMut;

pin_project_lite::pin_project! {
    pub struct FindMapAsync<I, F>
    where
        I: AsyncIterator,
        F: FnMut<(I::Item,)>,
        F: ?Sized,
        F::Output: IntoFuture,
        <F::Output as IntoFuture>::IntoFuture: OptionFuture,
    {
        #[pin]
        inner: FilterMapAsync<I, F>
    }
}

impl<I, F> FindMapAsync<I, F>
where
    I: AsyncIterator,
    F: FnMut<(I::Item,)>,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::IntoFuture: OptionFuture,
{
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self {
            inner: FilterMapAsync::new(iter, f),
        }
    }
}

impl<I, F> Clone for FindMapAsync<I, F>
where
    I: AsyncIterator + Clone,
    F: FnMut<(I::Item,)> + Clone,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::IntoFuture: OptionFuture + Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<I, F, T> Future for FindMapAsync<I, F>
where
    I: AsyncIterator,
    F: FnMut<(I::Item,)> + ?Sized,
    F::Output: IntoFuture<Output = Option<T>>,
{
    type Output = Option<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll_next(cx)
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iterator_ext::AsyncIteratorExt;
    use futures_util::future::{self, Ready};
    use futures_util::stream;

    fn find_3_then_mul_10(x: u32) -> Ready<Option<u64>> {
        future::ready((x == 3).then_some(u64::from(x) * 10))
    }

    fn find_7_then_mul_10(x: u32) -> Ready<Option<u64>> {
        future::ready((x == 7).then_some(u64::from(x) * 10))
    }

    #[tokio::test]
    async fn test_find_map_async() {
        let future = stream::iter([2, 3, 5]).slim_find_map_async(find_3_then_mul_10);

        assert_eq!(future.await, Some(30));
    }

    #[tokio::test]
    async fn test_find_map_async_fail() {
        let future = stream::iter([2, 3, 5]).slim_find_map_async(find_7_then_mul_10);

        assert!(future.await.is_none());
    }

    #[tokio::test]
    async fn test_find_map_async_clone() {
        let future = stream::iter([2, 3, 5]).slim_find_map_async(find_3_then_mul_10);
        let future_2 = future.clone();

        assert_eq!(future.await, Some(30));
        assert_eq!(future_2.await, Some(30));
    }
}
