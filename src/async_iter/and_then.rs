use crate::async_iter::map::Map;
use crate::support::fns::AndThenFn;
use crate::support::{AsyncIterator, FromResidual, FusedAsyncIterator, Try};
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::FnMut;

pin_project_lite::pin_project! {
    pub struct AndThen<I, F>
    where
        F: ?Sized
    {
        #[pin]
        inner: Map<I, AndThenFn<F>>,
    }
}

impl<I, F> AndThen<I, F> {
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self {
            inner: Map::new(iter, AndThenFn::new(f)),
        }
    }
}

impl<I, F> Clone for AndThen<I, F>
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

impl<I, F> AsyncIterator for AndThen<I, F>
where
    I: AsyncIterator,
    I::Item: Try,
    F: FnMut<(<I::Item as Try>::Output,)> + ?Sized,
    F::Output: FromResidual<<I::Item as Try>::Residual>,
{
    type Item = F::Output;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.project().inner.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<I, F> FusedAsyncIterator for AndThen<I, F>
where
    I: FusedAsyncIterator,
    I::Item: Try,
    F: FnMut<(<I::Item as Try>::Output,)> + ?Sized,
    F::Output: FromResidual<<I::Item as Try>::Residual>,
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

    fn and_then_fn(x: u32) -> Result<u64, u64> {
        if x < 12 {
            Ok(u64::from(x * 100))
        } else {
            Err(u64::from(x * 1000))
        }
    }

    #[tokio::test]
    async fn test_and_then() {
        let iter = stream::iter([Ok::<_, u32>(2), Ok(3), Err(5), Err(7), Ok(11), Ok(13)]).slim_and_then(and_then_fn);

        assert_eq!(
            iter.collect::<Vec<_>>().await,
            [Ok(200), Ok(300), Err(5), Err(7), Ok(1100), Err(13000)],
        );
    }

    #[tokio::test]
    async fn test_and_then_clone() {
        let iter = stream::iter([Ok::<_, u32>(2), Ok(3), Err(5), Err(7), Ok(11), Ok(13)]).slim_and_then(and_then_fn);
        let iter_2 = iter.clone();

        assert_eq!(
            iter.collect::<Vec<_>>().await,
            [Ok(200), Ok(300), Err(5), Err(7), Ok(1100), Err(13000)],
        );

        assert_eq!(
            iter_2.collect::<Vec<_>>().await,
            [Ok(200), Ok(300), Err(5), Err(7), Ok(1100), Err(13000)],
        );
    }
}
