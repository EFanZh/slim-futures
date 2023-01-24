use crate::async_iter::FilterAsync;
use crate::support::{AsyncIterator, PredicateFn};
use core::future::{Future, IntoFuture};
use core::pin::Pin;
use core::task::{Context, Poll};

pin_project_lite::pin_project! {
    pub struct FindAsync<I, P>
    where
        I: AsyncIterator,
        P: PredicateFn<I::Item>,
        P: ?Sized,
        <P as PredicateFn<I::Item>>::Output: IntoFuture,
    {
        #[pin]
        inner: FilterAsync<I, P>,
    }
}

impl<I, P> FindAsync<I, P>
where
    I: AsyncIterator,
    P: PredicateFn<I::Item>,
    <P as PredicateFn<I::Item>>::Output: IntoFuture,
{
    pub(crate) fn new(iter: I, predicate: P) -> Self {
        Self {
            inner: FilterAsync::new(iter, predicate),
        }
    }
}

impl<I, P> Clone for FindAsync<I, P>
where
    I: AsyncIterator + Clone,
    I::Item: Clone,
    P: PredicateFn<I::Item> + Clone,
    <P as PredicateFn<I::Item>>::Output: IntoFuture,
    <<P as PredicateFn<I::Item>>::Output as IntoFuture>::IntoFuture: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<I, P> Future for FindAsync<I, P>
where
    I: AsyncIterator,
    P: PredicateFn<I::Item> + ?Sized,
    <P as PredicateFn<I::Item>>::Output: IntoFuture<Output = bool>,
{
    type Output = Option<I::Item>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll_next(cx)
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iter_ext::AsyncIteratorExt;
    use futures_util::{future, stream};

    #[tokio::test]
    async fn test_find_async() {
        let future = stream::iter([2, 3, 5]).slim_find_async(|&x| future::ready(x > 2));

        assert_eq!(future.await, Some(3));
    }

    #[tokio::test]
    async fn test_find_async_fail() {
        let future = stream::iter([2, 3, 5]).slim_find_async(|&x| future::ready(x < 1));

        assert!(future.await.is_none());
    }

    #[tokio::test]
    async fn test_find_async_clone() {
        let future = stream::iter([2, 3, 5]).slim_find_async(|&x| future::ready(x > 2));
        let future_2 = future.clone();

        assert_eq!(future.await, Some(3));
        assert_eq!(future_2.await, Some(3));
    }
}
