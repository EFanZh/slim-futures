use crate::async_iter::and_then_async::AndThenAsync;
use crate::support::fns::MapOkAsyncFn;
use crate::support::{AsyncIterator, FusedAsyncIterator, Residual, Try};
use core::future::IntoFuture;
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::FnMut;
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    pub struct MapOkAsync<I, F>
    where
        I: AsyncIterator,
        I::Item: Try,
        <I::Item as Try>::Residual: Residual<<F::Output as IntoFuture>::Output>,
        F: FnMut<(<I::Item as Try>::Output,)>,
        F: ?Sized,
        F::Output: IntoFuture,
    {
        #[pin]
        inner: AndThenAsync<I, MapOkAsyncFn<F, <I::Item as Try>::Residual>>,
    }
}

impl<I, F> MapOkAsync<I, F>
where
    I: AsyncIterator,
    I::Item: Try,
    <I::Item as Try>::Residual: Residual<<F::Output as IntoFuture>::Output>,
    F: FnMut<(<I::Item as Try>::Output,)>,
    F::Output: IntoFuture,
{
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self {
            inner: AndThenAsync::new(iter, MapOkAsyncFn::new(f)),
        }
    }
}

impl<I, F> Clone for MapOkAsync<I, F>
where
    I: AsyncIterator + Clone,
    I::Item: Try,
    <I::Item as Try>::Residual: Residual<<F::Output as IntoFuture>::Output>,
    F: FnMut<(<I::Item as Try>::Output,)> + Clone,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::IntoFuture: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<I, F> AsyncIterator for MapOkAsync<I, F>
where
    I: AsyncIterator,
    I::Item: Try,
    <I::Item as Try>::Residual: Residual<<F::Output as IntoFuture>::Output>,
    F: FnMut<(<I::Item as Try>::Output,)> + ?Sized,
    F::Output: IntoFuture,
{
    type Item = <<I::Item as Try>::Residual as Residual<<F::Output as IntoFuture>::Output>>::TryType;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.project().inner.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<I, F> FusedAsyncIterator for MapOkAsync<I, F>
where
    I: FusedAsyncIterator,
    I::Item: Try,
    <I::Item as Try>::Residual: Residual<<F::Output as IntoFuture>::Output>,
    F: FnMut<(<I::Item as Try>::Output,)> + ?Sized,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::IntoFuture: FusedFuture,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iterator_ext::AsyncIteratorExt;
    use futures_util::future::Ready;
    use futures_util::{future, stream, StreamExt};
    use std::vec::Vec;

    fn map_ok_async_fn(x: u32) -> Ready<u64> {
        future::ready(u64::from(x * 100))
    }

    #[tokio::test]
    async fn test_map_ok_async() {
        let iter = stream::iter([Ok(2), Ok(3), Err(5), Err(7), Ok(11), Ok(13)]).slim_map_ok_async(map_ok_async_fn);

        assert_eq!(
            iter.collect::<Vec<_>>().await,
            [Ok(200), Ok(300), Err(5), Err(7), Ok(1100), Ok(1300)],
        );
    }

    #[tokio::test]
    async fn test_map_ok_async_with_option() {
        let iter = stream::iter([Some(2), Some(3), None, None, Some(5), Some(7)]).slim_map_ok_async(map_ok_async_fn);

        assert_eq!(
            iter.collect::<Vec<_>>().await,
            [Some(200), Some(300), None, None, Some(500), Some(700)],
        );
    }

    #[tokio::test]
    async fn test_map_ok_async_clone() {
        let iter = stream::iter([Ok(2), Ok(3), Err(5), Err(7), Ok(11), Ok(13)]).slim_map_ok_async(map_ok_async_fn);
        let iter_2 = iter.clone();

        assert_eq!(
            iter.collect::<Vec<_>>().await,
            [Ok(200), Ok(300), Err(5), Err(7), Ok(1100), Ok(1300)],
        );

        assert_eq!(
            iter_2.collect::<Vec<_>>().await,
            [Ok(200), Ok(300), Err(5), Err(7), Ok(1100), Ok(1300)],
        );
    }
}
