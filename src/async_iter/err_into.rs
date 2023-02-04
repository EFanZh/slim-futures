use crate::async_iter::map_err::MapErr;
use crate::support::{AsyncIterator, FusedAsyncIterator, ResultAsyncIterator};
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::fns::IntoFn;

pin_project_lite::pin_project! {
    pub struct ErrInto<I, E> {
        #[pin]
        inner: MapErr<I, IntoFn<E>>,
    }
}

impl<I, E> ErrInto<I, E> {
    pub(crate) fn new(iter: I) -> Self {
        Self {
            inner: MapErr::new(iter, IntoFn::default()),
        }
    }
}

impl<I, E> Clone for ErrInto<I, E>
where
    I: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<I, E> AsyncIterator for ErrInto<I, E>
where
    I: ResultAsyncIterator,
    I::Error: Into<E>,
{
    type Item = Result<I::Ok, E>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.project().inner.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<I, E> FusedAsyncIterator for ErrInto<I, E>
where
    I: ResultAsyncIterator + FusedAsyncIterator,
    I::Error: Into<E>,
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

    #[tokio::test]
    async fn test_err_into() {
        let iter = stream::iter([Ok(2), Err(3), Ok(5), Err(7)]).slim_err_into::<Option<_>>();

        assert_eq!(
            iter.collect::<Vec<_>>().await,
            [Ok(2), Err(Some(3)), Ok(5), Err(Some(7))]
        );
    }

    #[tokio::test]
    async fn test_err_into_clone() {
        let iter = stream::iter([Ok(2), Err(3), Ok(5), Err(7)]).slim_err_into();
        let iter_2 = iter.clone();

        assert_eq!(
            iter.collect::<Vec<_>>().await,
            [Ok(2), Err(Some(3)), Ok(5), Err(Some(7))]
        );

        assert_eq!(
            iter_2.collect::<Vec<_>>().await,
            [Ok(2), Err(Some(3)), Ok(5), Err(Some(7))],
        );
    }
}
