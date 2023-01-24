use crate::async_iter::Filter;
use crate::support::{AsyncIterator, FusedAsyncIterator};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::FnMut;
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    pub struct Find<I, P> {
        #[pin]
        inner: Filter<I, P>
    }
}

impl<I, P> Find<I, P> {
    pub(crate) fn new(iter: I, predicate: P) -> Self {
        Self {
            inner: Filter::new(iter, predicate),
        }
    }
}

impl<I, P> Clone for Find<I, P>
where
    I: Clone,
    P: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<I, P> Future for Find<I, P>
where
    I: AsyncIterator,
    P: for<'a> FnMut<(&'a I::Item,), Output = bool>,
{
    type Output = Option<I::Item>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll_next(cx)
    }
}

impl<I, P> FusedFuture for Find<I, P>
where
    I: FusedAsyncIterator,
    P: for<'a> FnMut<(&'a I::Item,), Output = bool>,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iter_ext::AsyncIteratorExt;
    use futures_util::stream;

    #[tokio::test]
    async fn test_find() {
        let future = stream::iter([2, 3, 5]).slim_find(|&x| x > 2);

        assert_eq!(future.await, Some(3));
    }

    #[tokio::test]
    async fn test_find_fail() {
        let future = stream::iter([2, 3, 5]).slim_find(|&x| x < 1);

        assert!(future.await.is_none());
    }

    #[tokio::test]
    async fn test_find_clone() {
        let future = stream::iter([2, 3, 5]).slim_find(|&x| x > 2);
        let future_2 = future.clone();

        assert_eq!(future.await, Some(3));
        assert_eq!(future_2.await, Some(3));
    }
}
