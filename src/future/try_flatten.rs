use crate::support::states::TwoPhases;
use crate::support::{FromResidual, Try};
use core::future::{Future, IntoFuture};
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    pub struct TryFlatten<Fut>
    where
        Fut: Future,
        Fut::Output: Try,
        <Fut::Output as Try>::Output: IntoFuture,
    {
        #[pin]
        inner: TwoPhases<Fut, <<Fut::Output as Try>::Output as IntoFuture>::IntoFuture>,
    }
}

impl<Fut> TryFlatten<Fut>
where
    Fut: Future,
    Fut::Output: Try,
    <Fut::Output as Try>::Output: IntoFuture,
{
    pub(crate) fn new(fut: Fut) -> Self {
        Self {
            inner: TwoPhases::First { state: fut },
        }
    }
}

impl<Fut> Clone for TryFlatten<Fut>
where
    Fut: Future + Clone,
    Fut::Output: Try,
    <Fut::Output as Try>::Output: IntoFuture,
    <<Fut::Output as Try>::Output as IntoFuture>::IntoFuture: Clone,
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
    <Fut::Output as Try>::Output: IntoFuture,
    <<Fut::Output as Try>::Output as IntoFuture>::Output: FromResidual<<Fut::Output as Try>::Residual> + Try,
{
    type Output = <<Fut::Output as Try>::Output as IntoFuture>::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        fn dispatch<T1, T2>(result: T1) -> ControlFlow<T2, <T1::Output as IntoFuture>::IntoFuture>
        where
            T1: Try,
            <T1 as Try>::Output: IntoFuture,
            T2: FromResidual<T1::Residual> + Try,
        {
            match result.branch() {
                ControlFlow::Continue(output) => ControlFlow::Continue(output.into_future()),
                ControlFlow::Break(residual) => ControlFlow::Break(T2::from_residual(residual)),
            }
        }

        self.project().inner.poll_with(
            cx,
            dispatch,
            <<Fut::Output as Try>::Output as IntoFuture>::IntoFuture::poll,
        )
    }
}

impl<Fut> FusedFuture for TryFlatten<Fut>
where
    Fut: FusedFuture,
    Fut::Output: Try,
    <Fut::Output as Try>::Output: IntoFuture,
    <<Fut::Output as Try>::Output as IntoFuture>::Output: FromResidual<<Fut::Output as Try>::Residual> + Try,
    <<Fut::Output as Try>::Output as IntoFuture>::IntoFuture: FusedFuture,
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
        let mut future = future::ok::<_, u32>(future::ok::<u32, u32>(2)).slim_try_flatten();

        assert!(!future.is_terminated());
        assert_eq!(future.by_ref().await, Ok(2));
        assert!(future.is_terminated());
    }

    #[tokio::test]
    async fn test_try_flatten_is_slim() {
        let make_base_future = || {
            crate::future::ok_by_copy(NonZeroU32::new(2).unwrap())
                .slim_map_ok(|_| crate::future::ok_by_copy::<_, ()>(()))
        };

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
