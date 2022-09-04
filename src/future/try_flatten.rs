use crate::support::{TryFuture, TwoPhases};
use futures_core::FusedFuture;
use std::future::Future;
use std::ops::ControlFlow;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    pub struct TryFlatten<Fut>
    where
        Fut: TryFuture,
    {
        #[pin]
        inner: TwoPhases<Fut, Fut::Ok>,
    }
}

impl<Fut, Fut2, E> TryFlatten<Fut>
where
    Fut: Future<Output = Result<Fut2, E>>,
{
    pub(crate) fn new(fut: Fut) -> Self {
        Self {
            inner: TwoPhases::First { state: fut },
        }
    }
}

impl<Fut, Fut2, E> Clone for TryFlatten<Fut>
where
    Fut: Clone + Future<Output = Result<Fut2, E>>,
    Fut2: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Fut, Fut2, E, T> Future for TryFlatten<Fut>
where
    Fut: Future<Output = Result<Fut2, E>>,
    Fut2: Future<Output = Result<T, E>>,
{
    type Output = Result<T, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll_with(
            cx,
            |fut, cx| match fut.poll(cx) {
                Poll::Ready(Ok(fut)) => ControlFlow::Continue(fut),
                Poll::Ready(Err(error)) => ControlFlow::Break(Poll::Ready(Err(error))),
                Poll::Pending => ControlFlow::Break(Poll::Pending),
            },
            Fut2::poll,
        )
    }
}

impl<Fut, Fut2, E, T> FusedFuture for TryFlatten<Fut>
where
    Fut: FusedFuture<Output = Result<Fut2, E>>,
    Fut2: FusedFuture<Output = Result<T, E>>,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_future_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::future::future_ext::FutureExt;
    use futures_core::FusedFuture;
    use futures_util::future::{self, Ready};

    #[tokio::test]
    async fn test_try_flatten() {
        assert_eq!(
            future::ready(Ok::<_, u32>(future::ok::<u32, _>(2)))
                .slim_try_flatten()
                .await,
            Ok(2),
        );

        assert_eq!(
            future::ready(Ok::<_, u32>(future::err::<u32, _>(2)))
                .slim_try_flatten()
                .await,
            Err(2),
        );

        assert_eq!(
            future::ready(Err::<Ready<Result<u32, _>>, u32>(2))
                .slim_try_flatten()
                .await,
            Err(2),
        );
    }

    #[tokio::test]
    async fn test_try_flatten_clone() {
        let future = future::ready(Ok::<_, u32>(future::ok::<u32, _>(2))).slim_try_flatten();
        let future_2 = future.clone();

        assert_eq!(future.await, Ok(2));
        assert_eq!(future_2.await, Ok(2));
    }

    #[tokio::test]
    async fn test_try_flatten_fused_future() {
        let mut future = future::ready(Ok::<_, u32>(future::ok::<u32, _>(2))).slim_try_flatten();

        assert!(!future.is_terminated());
        assert_eq!((&mut future).await, Ok(2));
        assert!(future.is_terminated());
    }
}
