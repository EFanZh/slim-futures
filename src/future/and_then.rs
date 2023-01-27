use crate::future::map::Map;
use crate::support::fns::AndThenFn;
use crate::support::{FromResidual, Try};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::FnMut;
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    #[derive(Clone)]
    pub struct AndThen<Fut, F> {
        #[pin]
        inner: Map<Fut, AndThenFn<F>>,
    }
}

impl<Fut, F> AndThen<Fut, F> {
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self {
            inner: Map::new(fut, AndThenFn::new(f)),
        }
    }
}

impl<Fut, F> Future for AndThen<Fut, F>
where
    Fut: Future,
    Fut::Output: Try,
    F: FnMut<(<Fut::Output as Try>::Output,)>,
    F::Output: FromResidual<<Fut::Output as Try>::Residual> + Try,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, F> FusedFuture for AndThen<Fut, F>
where
    Fut: FusedFuture,
    Fut::Output: Try,
    F: FnMut<(<Fut::Output as Try>::Output,)>,
    F::Output: FromResidual<<Fut::Output as Try>::Residual> + Try,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::future::future_ext::FutureExt;
    use futures_core::FusedFuture;
    use futures_util::future;
    use std::mem;

    #[allow(clippy::unnecessary_wraps)]
    fn ok_plus_3(value: u32) -> Result<u32, u32> {
        Ok(value + 3)
    }

    #[allow(clippy::unnecessary_wraps)]
    fn err_plus_3(value: u32) -> Result<u32, u32> {
        Err(value + 3)
    }

    #[tokio::test]
    async fn test_and_then() {
        assert_eq!(future::ok::<u32, u32>(2).slim_and_then(ok_plus_3).await, Ok(5));
        assert_eq!(future::ok::<u32, u32>(2).slim_and_then(err_plus_3).await, Err(5));
        assert_eq!(future::err::<u32, u32>(2).slim_and_then(ok_plus_3).await, Err(2));
        assert_eq!(future::err::<u32, u32>(2).slim_and_then(err_plus_3).await, Err(2));
    }

    #[tokio::test]
    async fn test_and_then_clone() {
        let future = future::ok::<u32, u32>(2).slim_and_then(ok_plus_3);
        let future_2 = future.clone();

        assert_eq!(future.await, Ok(5));
        assert_eq!(future_2.await, Ok(5));
    }

    #[tokio::test]
    async fn test_and_then_fused_future() {
        let mut future = future::ok::<u32, u32>(2).slim_and_then(ok_plus_3);

        assert!(!future.is_terminated());
        assert_eq!(future.by_ref().await, Ok(5));
        assert!(future.is_terminated());
    }

    #[tokio::test]
    async fn test_and_then_is_slim() {
        let make_base_future = || crate::future::ok_by_copy::<u32, u32>(2);
        let base_future = make_base_future();
        let future = make_base_future().slim_and_then(Ok::<u32, u32>);

        assert_eq!(mem::size_of_val(&base_future), mem::size_of_val(&future));
        assert_eq!(base_future.await, Ok(2));
        assert_eq!(future.await, Ok(2));
    }
}
