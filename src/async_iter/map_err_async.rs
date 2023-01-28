use crate::async_iter::or_else_async::OrElseAsync;
use crate::support::fns::MapErrAsyncFn;
use crate::support::{AsyncIterator, FusedAsyncIterator, ResultAsyncIterator};
use core::future::IntoFuture;
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::FnMut;
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    pub struct MapErrAsync<I, F>
    where
        I: ResultAsyncIterator,
        F: FnMut<(I::Error,)>,
        F: ?Sized,
        F::Output: IntoFuture,
    {
        #[pin]
        inner: OrElseAsync<I, MapErrAsyncFn<F, I::Ok>>,
    }
}

impl<I, F> MapErrAsync<I, F>
where
    I: ResultAsyncIterator,
    F: FnMut<(I::Error,)>,
    F::Output: IntoFuture,
{
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self {
            inner: OrElseAsync::new(iter, MapErrAsyncFn::new(f)),
        }
    }
}

impl<I, F> Clone for MapErrAsync<I, F>
where
    I: ResultAsyncIterator + Clone,
    F: FnMut<(I::Error,)> + Clone,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::IntoFuture: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<I, F> AsyncIterator for MapErrAsync<I, F>
where
    I: ResultAsyncIterator,
    F: FnMut<(I::Error,)> + ?Sized,
    F::Output: IntoFuture,
{
    type Item = Result<I::Ok, <F::Output as IntoFuture>::Output>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.project().inner.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<I, F> FusedAsyncIterator for MapErrAsync<I, F>
where
    I: ResultAsyncIterator + FusedAsyncIterator,
    F: FnMut<(I::Error,)> + ?Sized,
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

    fn map_err_async_fn(x: u32) -> Ready<u64> {
        future::ready(u64::from(x * 100))
    }

    #[tokio::test]
    async fn test_map_err_async() {
        let iter = stream::iter([Ok(2), Ok(3), Err(5), Err(7), Ok(11), Ok(13)]).slim_map_err_async(map_err_async_fn);

        assert_eq!(
            iter.collect::<Vec<_>>().await,
            [Ok(2), Ok(3), Err(500), Err(700), Ok(11), Ok(13)],
        );
    }

    #[tokio::test]
    async fn test_map_err_async_clone() {
        let iter = stream::iter([Ok(2), Ok(3), Err(5), Err(7), Ok(11), Ok(13)]).slim_map_err_async(map_err_async_fn);
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
