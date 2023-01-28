use crate::async_iter::map::Map;
use crate::support::fns::MapErrFn;
use crate::support::{AsyncIterator, FusedAsyncIterator, ResultAsyncIterator};
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::FnMut;

pin_project_lite::pin_project! {
    pub struct MapErr<I, F>
    where
        F: ?Sized
    {
        #[pin]
        inner: Map<I, MapErrFn<F>>,
    }
}

impl<I, F> MapErr<I, F> {
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self {
            inner: Map::new(iter, MapErrFn::new(f)),
        }
    }
}

impl<I, F> Clone for MapErr<I, F>
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

impl<I, F> AsyncIterator for MapErr<I, F>
where
    I: ResultAsyncIterator,
    F: FnMut<(I::Error,)> + ?Sized,
{
    type Item = Result<I::Ok, F::Output>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.project().inner.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<I, F> FusedAsyncIterator for MapErr<I, F>
where
    I: ResultAsyncIterator + FusedAsyncIterator,
    F: FnMut<(I::Error,)> + ?Sized,
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

    fn map_err_fn(x: u32) -> u64 {
        u64::from(x * 100)
    }

    #[tokio::test]
    async fn test_map_err() {
        let iter = stream::iter([Ok(2), Ok(3), Err(5), Err(7), Ok(11), Ok(13)]).slim_map_err(map_err_fn);

        assert_eq!(
            iter.collect::<Vec<_>>().await,
            [Ok(2), Ok(3), Err(500), Err(700), Ok(11), Ok(13)],
        );
    }

    #[tokio::test]
    async fn test_map_err_clone() {
        let iter = stream::iter([Ok(2), Ok(3), Err(5), Err(7), Ok(11), Ok(13)]).slim_map_err(map_err_fn);
        let iter_2 = iter.clone();

        assert_eq!(
            iter.collect::<Vec<_>>().await,
            [Ok(2), Ok(3), Err(500), Err(700), Ok(11), Ok(13)],
        );

        assert_eq!(
            iter_2.collect::<Vec<_>>().await,
            [Ok(2), Ok(3), Err(500), Err(700), Ok(11), Ok(13)],
        );
    }
}
