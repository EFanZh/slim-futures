use crate::support::{FromResidual, Try, TwoPhases};
use core::future::Future;
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    pub struct TryFlatten<Fut>
    where
        Fut: Future,
        Fut::Output: Try
    {
        #[pin]
        inner: TwoPhases<Fut, <Fut::Output as Try>::Output>,
    }
}

impl<Fut> TryFlatten<Fut>
where
    Fut: Future,
    Fut::Output: Try,
{
    pub(crate) fn new(fut: Fut) -> Self {
        Self {
            inner: TwoPhases::First { state: fut },
        }
    }
}

impl<Fut> Clone for TryFlatten<Fut>
where
    Fut: Clone + Future,
    Fut::Output: Try,
    <Fut::Output as Try>::Output: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Fut> Future for TryFlatten<Fut>
where
    Fut: Future,
    Fut::Output: Try,
    <Fut::Output as Try>::Output: Future,
    <<Fut::Output as Try>::Output as Future>::Output: FromResidual<<Fut::Output as Try>::Residual> + Try,
{
    type Output = <<Fut::Output as Try>::Output as Future>::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        fn dispatch<T1, T2>(result: T1) -> ControlFlow<T2, T1::Output>
        where
            T1: Try,
            T2: FromResidual<T1::Residual> + Try,
        {
            match result.branch() {
                ControlFlow::Continue(output) => ControlFlow::Continue(output),
                ControlFlow::Break(residual) => ControlFlow::Break(T2::from_residual(residual)),
            }
        }

        self.project()
            .inner
            .poll_with(cx, dispatch, <Fut::Output as Try>::Output::poll)
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
    use crate::test_utilities::Yield;
    use futures_core::FusedFuture;
    use futures_util::future::{self, Ready};
    use futures_util::TryFutureExt;
    use std::mem;
    use std::num::NonZeroU32;

    #[tokio::test]
    async fn test_try_flatten() {
        assert_eq!(
            future::ok::<_, u32>(future::ok::<u32, u32>(2)).slim_try_flatten().await,
            Ok(2),
        );

        assert_eq!(
            future::ok::<_, u32>(future::err::<u32, u32>(2))
                .slim_try_flatten()
                .await,
            Err(2),
        );

        assert_eq!(
            future::err::<Ready<Result<u32, u32>>, u32>(2).slim_try_flatten().await,
            Err(2),
        );
    }

    #[tokio::test]
    async fn test_try_flatten_with_pending() {
        let future = Yield::new(1)
            .slim_map(|()| Ok::<_, u32>(future::ok::<u32, u32>(2)))
            .slim_try_flatten();

        assert_eq!(future.await, Ok(2));
    }

    #[tokio::test]
    async fn test_try_flatten_clone() {
        let future = future::ok::<_, u32>(future::ok::<u32, u32>(2)).slim_try_flatten();
        let future_2 = future.clone();

        assert_eq!(future.await, Ok(2));
        assert_eq!(future_2.await, Ok(2));
    }

    #[tokio::test]
    async fn test_try_flatten_fused_future() {
        let mut future = future::ok::<_, u32>(future::ok::<u32, _>(2)).slim_try_flatten();

        assert!(!future.is_terminated());
        assert_eq!(future.by_ref().await, Ok(2));
        assert!(future.is_terminated());
    }

    #[tokio::test]
    async fn test_try_flatten_is_slim() {
        let make_base_future =
            || crate::future::ok(NonZeroU32::new(2).unwrap()).slim_map_ok(|_| crate::future::ok::<_, ()>(()));

        let base_future = make_base_future();
        let future_1 = make_base_future().slim_try_flatten();
        let future_2 = make_base_future().try_flatten();

        assert_eq!(mem::size_of_val(&base_future), mem::size_of_val(&future_1));
        assert!(mem::size_of_val(&future_1) < mem::size_of_val(&future_2));
        assert_eq!(base_future.await.unwrap().await, Ok(()));
        assert_eq!(future_1.await, Ok(()));
        assert_eq!(future_2.await, Ok(()));
    }
}
