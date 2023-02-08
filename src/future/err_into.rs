use crate::future::map_err::MapErr;
use crate::support::ResultFuture;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::fns::IntoFn;
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    pub struct ErrInto<Fut, T> {
        #[pin]
        inner: MapErr<Fut, IntoFn<T>>,
    }
}

impl<Fut, T> ErrInto<Fut, T> {
    pub(crate) fn new(fut: Fut) -> Self {
        Self {
            inner: MapErr::new(fut, IntoFn::default()),
        }
    }
}

impl<Fut, T> Clone for ErrInto<Fut, T>
where
    Fut: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Fut, T> Future for ErrInto<Fut, T>
where
    Fut: ResultFuture,
    Fut::Error: Into<T>,
{
    type Output = Result<Fut::Ok, T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, T> FusedFuture for ErrInto<Fut, T>
where
    Fut: ResultFuture + FusedFuture,
    Fut::Error: Into<T>,
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
    use futures_util::{future, TryFutureExt};
    use std::mem;

    #[tokio::test]
    async fn test_err_into() {
        assert_eq!(future::ok::<u32, u32>(7).slim_err_into::<Option<_>>().await, Ok(7));

        assert_eq!(
            future::err::<u32, u32>(7).slim_err_into::<Option<_>>().await,
            Err(Some(7))
        );
    }

    #[tokio::test]
    async fn test_err_into_clone() {
        let future = future::err::<u32, u32>(7).slim_err_into::<Option<_>>();
        let future_2 = future.clone();

        assert_eq!(future.await, Err(Some(7)));
        assert_eq!(future_2.await, Err(Some(7)));
    }

    #[tokio::test]
    async fn test_err_into_fused_future() {
        let mut future = future::err::<u32, u32>(7).slim_err_into::<Option<_>>();

        assert!(!future.is_terminated());
        assert_eq!(future.by_ref().await, Err(Some(7)));
        assert!(future.is_terminated());
    }

    #[tokio::test]
    async fn test_err_into_is_slim() {
        let make_base_future = || err::err_by_copy::<u32, u32>(2);
        let base_future = make_base_future();
        let future_1 = make_base_future().slim_err_into::<Option<_>>();
        let future_2 = make_base_future().err_into::<Option<_>>();

        assert_eq!(mem::size_of_val(&base_future), mem::size_of_val(&future_1));
        assert!(mem::size_of_val(&future_1) < mem::size_of_val(&future_2));
        assert_eq!(base_future.await, Err(2));
        assert_eq!(future_1.await, Err(Some(2)));
        assert_eq!(future_2.await, Err(Some(2)));
    }
}
