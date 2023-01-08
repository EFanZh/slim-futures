use crate::support::{AsyncIterator, FusedAsyncIterator, TwoPhases};
use core::future::Future;
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::FusedFuture;

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
        self.project().inner.poll_with(cx, ControlFlow::Continue, f)
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
    use crate::test_utilities::Yield;
    use futures_core::FusedFuture;
    use futures_util::{future, stream, FutureExt as _, StreamExt};
    use std::mem;
    use std::num::NonZeroU32;

    fn make_flatten_future() -> impl FusedFuture<Output = u32> + Clone {
        Yield::new(1).slim_map(|()| future::ready(2)).slim_flatten()
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
        assert_eq!(future.by_ref().await, 2);
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
        let make_base_future =
            || crate::future::ready(NonZeroU32::new(2).unwrap()).slim_map(|_| crate::future::ready(()));

        let base_future = make_base_future();
        let future_1 = make_base_future().slim_flatten();
        let future_2 = make_base_future().flatten();

        assert_eq!(mem::size_of_val(&base_future), mem::size_of_val(&future_1));
        assert!(mem::size_of_val(&future_1) < mem::size_of_val(&future_2));
        assert!(matches!(base_future.await.await, ()));
        assert!(matches!(future_1.await, ()));
        assert!(matches!(future_2.await, ()));
    }
}
