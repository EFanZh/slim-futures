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
    use crate::test_utilities::{self, Defer};
    use futures_core::FusedFuture;
    use futures_util::{future, stream, FutureExt as _, StreamExt};
    use std::mem;
    use std::task::Poll;

    #[tokio::test]
    async fn test_flatten() {
        let original = future::ready(future::ready(7));
        let wrapped = original.clone().slim_flatten();

        assert_eq!(original.await.await, 7);
        assert_eq!(wrapped.await, 7);
    }

    #[tokio::test]
    async fn test_flatten_future_first_pending() {
        let original = Defer::new(1).slim_map(|()| future::ready(7));
        let wrapped = original.clone().slim_flatten();

        assert_eq!(original.await.await, 7);
        assert_eq!(wrapped.await, 7);
    }

    #[tokio::test]
    async fn test_flatten_clone() {
        let mut wrapped = future::ready(Defer::new(1)).slim_flatten();

        assert_eq!(futures_util::poll!(wrapped.clone()), Poll::Pending);

        assert!(futures_util::poll!(&mut wrapped).is_pending());

        assert_eq!(futures_util::poll!(wrapped.clone()), Poll::Ready(()));
    }

    #[tokio::test]
    async fn test_flatten_fused_future() {
        let mut wrapped = future::ready(Defer::new(1)).slim_flatten();

        assert!(!wrapped.is_terminated());

        assert!(futures_util::poll!(&mut wrapped).is_pending());

        assert!(!wrapped.is_terminated());

        assert_eq!(futures_util::poll!(&mut wrapped), Poll::Ready(()));

        assert!(wrapped.is_terminated());
    }

    #[tokio::test]
    async fn test_flatten_async_iter() {
        let original = future::ready(stream::iter(Some(7)));
        let mut wrapped = original.clone().slim_flatten_async_iter();

        let mut original_async_iter = original.await;

        assert_eq!(original_async_iter.next().await, Some(7));
        assert_eq!(original_async_iter.next().await, None);

        assert_eq!(wrapped.next().await, Some(7));
        assert_eq!(wrapped.next().await, None);
    }

    #[test]
    fn test_flatten_is_slim() {
        let make_future = || crate::future::lazy(|_| test_utilities::almost_full_bytes_future());
        let future = make_future().slim_flatten();
        let other = make_future().flatten();

        assert!(mem::size_of_val(&future) < mem::size_of_val(&other));
    }
}
