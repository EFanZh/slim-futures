use crate::async_iter::map::Map;
use crate::support::fns::MapOkFn;
use crate::support::{AsyncIterator, FusedAsyncIterator, Residual, Try};
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::FnMut;

pin_project_lite::pin_project! {
    pub struct MapOk<I, F>
    where
        F: ?Sized
    {
        #[pin]
        inner: Map<I, MapOkFn<F>>,
    }
}

impl<I, F> MapOk<I, F> {
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self {
            inner: Map::new(iter, MapOkFn::new(f)),
        }
    }
}

impl<I, F> Clone for MapOk<I, F>
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

impl<I, F> AsyncIterator for MapOk<I, F>
where
    I: AsyncIterator,
    I::Item: Try,
    <I::Item as Try>::Residual: Residual<F::Output>,
    F: FnMut<(<I::Item as Try>::Output,)> + ?Sized,
{
    type Item = <<I::Item as Try>::Residual as Residual<F::Output>>::TryType;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.project().inner.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<I, F> FusedAsyncIterator for MapOk<I, F>
where
    I: FusedAsyncIterator,
    I::Item: Try,
    <I::Item as Try>::Residual: Residual<F::Output>,
    F: FnMut<(<I::Item as Try>::Output,)> + ?Sized,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iterator_ext::AsyncIteratorExt;
    use futures_util::{stream, StreamExt};
    use std::vec::Vec;

    fn map_ok_fn(x: u32) -> u64 {
        u64::from(x * 100)
    }

    #[tokio::test]
    async fn test_map_ok() {
        let iter = stream::iter([Ok(2), Ok(3), Err(5), Err(7), Ok(11), Ok(13)]).slim_map_ok(map_ok_fn);

        assert_eq!(
            iter.collect::<Vec<_>>().await,
            [Ok(200), Ok(300), Err(5), Err(7), Ok(1100), Ok(1300)],
        );
    }

    #[tokio::test]
    async fn test_map_ok_clone() {
        let iter = stream::iter([Ok(2), Ok(3), Err(5), Err(7), Ok(11), Ok(13)]).slim_map_ok(map_ok_fn);
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
