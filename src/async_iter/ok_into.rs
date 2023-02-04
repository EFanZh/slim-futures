use crate::async_iter::map_ok::MapOk;
use crate::support::{AsyncIterator, FusedAsyncIterator, Residual, Try};
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::fns::IntoFn;

pin_project_lite::pin_project! {
    pub struct OkInto<I, T> {
        #[pin]
        inner: MapOk<I, IntoFn<T>>,
    }
}

impl<I, T> OkInto<I, T> {
    pub(crate) fn new(iter: I) -> Self {
        Self {
            inner: MapOk::new(iter, IntoFn::default()),
        }
    }
}

impl<I, T> Clone for OkInto<I, T>
where
    I: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<I, T> AsyncIterator for OkInto<I, T>
where
    I: AsyncIterator,
    I::Item: Try,
    <I::Item as Try>::Output: Into<T>,
    <I::Item as Try>::Residual: Residual<T>,
{
    type Item = <<I::Item as Try>::Residual as Residual<T>>::TryType;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.project().inner.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<I, T> FusedAsyncIterator for OkInto<I, T>
where
    I: FusedAsyncIterator,
    I::Item: Try,
    <I::Item as Try>::Output: Into<T>,
    <I::Item as Try>::Residual: Residual<T>,
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
    async fn test_ok_into() {
        let iter = stream::iter([Ok(2), Err(3), Ok(5), Err(7)]).slim_ok_into::<Option<_>>();

        assert_eq!(
            iter.collect::<Vec<_>>().await,
            [Ok(Some(2)), Err(3), Ok(Some(5)), Err(7)],
        );
    }

    #[tokio::test]
    async fn test_ok_into_clone() {
        let iter = stream::iter([Ok(2), Err(3), Ok(5), Err(7)]).slim_ok_into();
        let iter_2 = iter.clone();

        assert_eq!(
            iter.collect::<Vec<_>>().await,
            [Ok(Some(2)), Err(3), Ok(Some(5)), Err(7)],
        );

        assert_eq!(
            iter_2.collect::<Vec<_>>().await,
            [Ok(Some(2)), Err(3), Ok(Some(5)), Err(7)],
        );
    }
}
