use crate::future::map::Map;
use crate::support::fns::OrElseFn;
use crate::support::{ResultFuture, Try};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::FnMut;
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    #[derive(Clone)]
    pub struct OrElse<Fut, F>
    where
        F: ?Sized,
    {
        #[pin]
        inner: Map<Fut, OrElseFn<F>>,
    }
}

impl<Fut, F> OrElse<Fut, F> {
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self {
            inner: Map::new(fut, OrElseFn::new(f)),
        }
    }
}

impl<Fut, F> Future for OrElse<Fut, F>
where
    Fut: ResultFuture,
    F: FnMut<(Fut::Error,)> + ?Sized,
    F::Output: Try<Output = Fut::Ok>,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, F> FusedFuture for OrElse<Fut, F>
where
    Fut: ResultFuture + FusedFuture,
    F: FnMut<(Fut::Error,)> + ?Sized,
    F::Output: Try<Output = Fut::Ok>,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::future::err;
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
    async fn test_or_else() {
        assert_eq!(future::ok::<u32, _>(2).slim_or_else(ok_plus_3).await, Ok(2));
        assert_eq!(future::ok::<u32, _>(2).slim_or_else(err_plus_3).await, Ok(2));
        assert_eq!(future::err::<u32, _>(2).slim_or_else(ok_plus_3).await, Ok(5));
        assert_eq!(future::err::<u32, _>(2).slim_or_else(err_plus_3).await, Err(5));
    }

    #[tokio::test]
    async fn test_or_else_with_option() {
        assert_eq!(future::err::<u32, _>(2).slim_or_else(|x| Some(x + 3)).await, Some(5),);
    }

    #[tokio::test]
    async fn test_or_else_clone() {
        let future = future::err::<u32, _>(2).slim_or_else(err_plus_3);
        let future_2 = future.clone();

        assert_eq!(future.await, Err(5));
        assert_eq!(future_2.await, Err(5));
    }

    #[tokio::test]
    async fn test_or_else_fused_future() {
        let mut future = future::err::<u32, _>(2).slim_or_else(err_plus_3);

        assert!(!future.is_terminated());
        assert_eq!(future.by_ref().await, Err(5));
        assert!(future.is_terminated());
    }

    #[tokio::test]
    async fn test_or_else_is_slim() {
        let make_base_future = || err::err_by_copy::<u32, _>(2);
        let base_future = make_base_future();
        let future = make_base_future().slim_or_else(err_plus_3);

        assert_eq!(mem::size_of_val(&base_future), mem::size_of_val(&future));
        assert_eq!(base_future.await, Err(2));
        assert_eq!(future.await, Err(5));
    }
}
