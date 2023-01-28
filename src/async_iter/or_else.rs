use crate::async_iter::map::Map;
use crate::support::fns::OrElseFn;
use crate::support::{AsyncIterator, FusedAsyncIterator, ResultAsyncIterator, Try};
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::FnMut;

pin_project_lite::pin_project! {
    pub struct OrElse<I, F>
    where
        F: ?Sized
    {
        #[pin]
        inner: Map<I, OrElseFn<F>>,
    }
}

impl<I, F> OrElse<I, F> {
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self {
            inner: Map::new(iter, OrElseFn::new(f)),
        }
    }
}

impl<I, F> Clone for OrElse<I, F>
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

impl<I, F> AsyncIterator for OrElse<I, F>
where
    I: ResultAsyncIterator,
    F: FnMut<(I::Error,)> + ?Sized,
    F::Output: Try<Output = I::Ok>,
{
    type Item = F::Output;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.project().inner.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<I, F> FusedAsyncIterator for OrElse<I, F>
where
    I: ResultAsyncIterator + FusedAsyncIterator,
    F: FnMut<(I::Error,)> + ?Sized,
    F::Output: Try<Output = I::Ok>,
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

    fn or_else_fn(x: u32) -> Result<u64, u64> {
        if x < 7 {
            Ok(u64::from(x * 100))
        } else {
            Err(u64::from(x * 1000))
        }
    }

    #[tokio::test]
    async fn test_or_else() {
        let iter = stream::iter([Ok(2), Ok(3), Err(5_u32), Err(7), Ok(11), Ok(13)]).slim_or_else(or_else_fn);

        assert_eq!(
            iter.collect::<Vec<_>>().await,
            [Ok(2), Ok(3), Ok(500), Err(7000), Ok(11), Ok(13)],
        );
    }

    #[tokio::test]
    async fn test_or_else_clone() {
        let iter = stream::iter([Ok(2), Ok(3), Err(5_u32), Err(7), Ok(11), Ok(13)]).slim_or_else(or_else_fn);
        let iter_2 = iter.clone();

        assert_eq!(
            iter.collect::<Vec<_>>().await,
            [Ok(2), Ok(3), Ok(500), Err(7000), Ok(11), Ok(13)],
        );

        assert_eq!(
            iter_2.collect::<Vec<_>>().await,
            [Ok(2), Ok(3), Ok(500), Err(7000), Ok(11), Ok(13)],
        );
    }
}
