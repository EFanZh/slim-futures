use crate::support::{TryFuture, TwoPhases};
use futures_core::FusedFuture;
use std::future::Future;
use std::ops::ControlFlow;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    pub struct TryFlattenErr<Fut>
    where
        Fut: TryFuture,
    {
        #[pin]
        inner: TwoPhases<Fut, Fut::Error>,
    }
}

impl<Fut, Fut2, T> TryFlattenErr<Fut>
where
    Fut: Future<Output = Result<T, Fut2>>,
{
    pub(crate) fn new(fut: Fut) -> Self {
        Self {
            inner: TwoPhases::First { state: fut },
        }
    }
}

impl<Fut, Fut2, T> Clone for TryFlattenErr<Fut>
where
    Fut: Clone + Future<Output = Result<T, Fut2>>,
    Fut2: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Fut, Fut2, T, E> Future for TryFlattenErr<Fut>
where
    Fut: Future<Output = Result<T, Fut2>>,
    Fut2: Future<Output = Result<T, E>>,
{
    type Output = Result<T, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll_with(
            cx,
            |fut, cx| match fut.poll(cx) {
                Poll::Ready(Ok(value)) => ControlFlow::Break(Poll::Ready(Ok(value))),
                Poll::Ready(Err(fut)) => ControlFlow::Continue(fut),
                Poll::Pending => ControlFlow::Break(Poll::Pending),
            },
            Fut2::poll,
        )
    }
}

impl<Fut, Fut2, T, E> FusedFuture for TryFlattenErr<Fut>
where
    Fut: FusedFuture<Output = Result<T, Fut2>>,
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
    async fn test_try_flatten_err() {
        assert_eq!(
            future::ready(Ok::<u32, Ready<Result<_, u32>>>(2))
                .slim_try_flatten_err()
                .await,
            Ok(2),
        );

        assert_eq!(
            future::ready(Err::<u32, _>(future::ok::<_, u32>(2)))
                .slim_try_flatten_err()
                .await,
            Ok(2),
        );

        assert_eq!(
            future::ready(Err::<u32, _>(future::err::<_, u32>(2)))
                .slim_try_flatten_err()
                .await,
            Err(2),
        );
    }

    #[tokio::test]
    async fn test_try_flatten_err_clone() {
        let future = future::ready(Err::<u32, _>(future::err::<_, u32>(2))).slim_try_flatten_err();
        let future_2 = future.clone();

        assert_eq!(future.await, Err(2));
        assert_eq!(future_2.await, Err(2));
    }

    #[tokio::test]
    async fn test_try_flatten_err_fused_future() {
        let mut future = future::ready(Err::<u32, _>(future::err::<_, u32>(2))).slim_try_flatten_err();

        assert!(!future.is_terminated());
        assert_eq!((&mut future).await, Err(2));
        assert!(future.is_terminated());
    }
}
