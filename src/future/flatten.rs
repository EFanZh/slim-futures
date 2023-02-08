use crate::support::states::TwoPhases;
use core::future::{Future, IntoFuture};
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    pub struct Flatten<Fut>
    where
        Fut: Future,
        Fut::Output: IntoFuture,
    {
        #[pin]
        inner: TwoPhases<Fut, <Fut::Output as IntoFuture>::IntoFuture>,
    }
}

impl<Fut> Flatten<Fut>
where
    Fut: Future,
    Fut::Output: IntoFuture,
{
    pub(crate) fn new(fut: Fut) -> Self {
        Self {
            inner: TwoPhases::new(fut),
        }
    }
}

impl<Fut> Clone for Flatten<Fut>
where
    Fut: Future + Clone,
    Fut::Output: IntoFuture,
    <Fut::Output as IntoFuture>::IntoFuture: Clone,
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
    Fut::Output: IntoFuture,
{
    type Output = <Fut::Output as IntoFuture>::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        fn dispatch<Fut>(fut: Fut) -> ControlFlow<Fut::Output, Fut::IntoFuture>
        where
            Fut: IntoFuture,
        {
            ControlFlow::Continue(fut.into_future())
        }

        self.project()
            .inner
            .poll_with(cx, dispatch, <Fut::Output as IntoFuture>::IntoFuture::poll)
    }
}

impl<Fut> FusedFuture for Flatten<Fut>
where
    Fut: FusedFuture,
    Fut::Output: IntoFuture,
    <Fut::Output as IntoFuture>::IntoFuture: FusedFuture,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_future_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::future::future_ext::FutureExt;
    use crate::future::ready;
    use crate::test_utilities::Yield;
    use futures_core::FusedFuture;
    use futures_util::{future, FutureExt as _};
    use std::mem;
    use std::num::NonZeroU32;

    fn make_flatten_future() -> impl FusedFuture<Output = u32> + Clone {
        Yield::new(1).slim_map(|()| future::ready(2)).slim_flatten()
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
    async fn test_flatten_is_slim() {
        let make_base_future =
            || ready::ready_by_copy(NonZeroU32::new(2).unwrap()).slim_map(|_| ready::ready_by_copy(()));

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
