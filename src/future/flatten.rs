use crate::support::{AsyncIterator, FusedAsyncIterator, TwoPhases};
use futures_core::FusedFuture;
use std::future::Future;
use std::ops::ControlFlow;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    pub struct Flatten<Fut>
    where
        Fut: Future,
    {
        #[pin]
        inner: TwoPhases<Fut, Fut::Output>,
    }
}

impl<Fut> Flatten<Fut>
where
    Fut: Future,
{
    pub(crate) fn new(fut: Fut) -> Self {
        Self {
            inner: TwoPhases::First { state: fut },
        }
    }

    fn poll_with<T>(
        self: Pin<&mut Self>,
        cx: &mut Context,
        f: impl FnOnce(Pin<&mut Fut::Output>, &mut Context) -> Poll<T>,
    ) -> Poll<T> {
        self.project().inner.poll_with(
            cx,
            |fut, cx| match fut.poll(cx) {
                Poll::Ready(fut) => ControlFlow::Continue(fut),
                Poll::Pending => ControlFlow::Break(Poll::Pending),
            },
            f,
        )
    }
}

impl<Fut> Clone for Flatten<Fut>
where
    Fut: Future + Clone,
    Fut::Output: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Fut> Future for Flatten<Fut>
where
    Fut: Future,
    Fut::Output: Future,
{
    type Output = <Fut::Output as Future>::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.poll_with(cx, Fut::Output::poll)
    }
}

impl<Fut> FusedFuture for Flatten<Fut>
where
    Fut: FusedFuture,
    Fut::Output: FusedFuture,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_future_terminated()
    }
}

impl<Fut> AsyncIterator for Flatten<Fut>
where
    Fut: Future,
    Fut::Output: AsyncIterator,
{
    type Item = <Fut::Output as AsyncIterator>::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.poll_with(cx, Fut::Output::poll_next)
    }
}

impl<Fut> FusedAsyncIterator for Flatten<Fut>
where
    Fut: FusedFuture,
    Fut::Output: FusedAsyncIterator,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_async_iter_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::future::future_ext::FutureExt;
    use crate::support::FusedAsyncIterator;
    use crate::test_utilities::{self, Defer};
    use futures_core::FusedFuture;
    use futures_util::{future, stream, FutureExt as _, StreamExt};
    use std::mem;

    fn make_flatten_future() -> impl FusedFuture<Output = u32> + Clone {
        Defer::new(1).slim_map(|()| future::ready(2)).slim_flatten()
    }

    fn make_flatten_async_iter() -> impl FusedAsyncIterator<Item = u32> {
        future::ready(stream::once(future::ready(2))).slim_flatten_async_iter()
    }

    #[tokio::test]
    async fn test_flatten() {
        assert_eq!(make_flatten_future().await, 2);
    }

    #[tokio::test]
    async fn test_flatten_clone() {
        let future = make_flatten_future();
        let future_2 = future.clone();

        assert_eq!(future.await, 2);
        assert_eq!(future_2.await, 2);
    }

    #[tokio::test]
    async fn test_flatten_fused_future() {
        let mut future = make_flatten_future();

        assert!(!future.is_terminated());
        assert_eq!((&mut future).await, 2);
        assert!(future.is_terminated());
    }

    #[tokio::test]
    async fn test_flatten_async_iter() {
        let mut iter = make_flatten_async_iter();

        assert_eq!(iter.next().await, Some(2));
        assert_eq!(iter.next().await, None);
    }

    #[tokio::test]
    async fn test_flatten_fused_async_iter() {
        let mut iter = make_flatten_async_iter();

        assert!(!iter.is_terminated());
        assert_eq!(iter.next().await, Some(2));
        assert!(iter.is_terminated());
        assert_eq!(iter.next().await, None);
        assert!(iter.is_terminated());
    }

    #[tokio::test]
    async fn test_flatten_is_slim() {
        let base_future = || crate::future::lazy(|_| test_utilities::almost_full_bytes_future(2));
        let future = base_future().slim_flatten();
        let future_2 = base_future().flatten();

        assert!(mem::size_of_val(&future) < mem::size_of_val(&future_2));
        assert_eq!(future.await, 2);
        assert_eq!(future_2.await, 2);
    }
}
