use crate::future::map_ok::MapOk;
use crate::support::fns::IntoFn;
use crate::support::ResultFuture;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    pub struct OkInto<Fut, U>
    where
        Fut: ResultFuture,
    {
        #[pin]
        inner: MapOk<Fut, IntoFn<Fut::Ok, U>>,
    }
}

impl<Fut, U> OkInto<Fut, U>
where
    Fut: ResultFuture,
{
    pub(crate) fn new(fut: Fut) -> Self {
        Self {
            inner: MapOk::new(fut, IntoFn::default()),
        }
    }
}

impl<Fut, U> Clone for OkInto<Fut, U>
where
    Fut: ResultFuture + Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Fut, U, T, E> Future for OkInto<Fut, U>
where
    Fut: Future<Output = Result<T, E>>,
    T: Into<U>,
{
    type Output = Result<U, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, E, T, U> FusedFuture for OkInto<Fut, U>
where
    Fut: FusedFuture<Output = Result<T, E>>,
    T: Into<U>,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::future::future_ext::FutureExt;
    use futures_core::FusedFuture;
    use futures_util::{future, TryFutureExt};
    use std::mem;

    #[tokio::test]
    async fn test_ok_into() {
        assert_eq!(future::ok::<u32, u32>(7).slim_ok_into::<Option<_>>().await, Ok(Some(7)));
        assert_eq!(future::err::<u32, u32>(7).slim_ok_into::<Option<_>>().await, Err(7));
    }

    #[tokio::test]
    async fn test_ok_into_clone() {
        let future = future::ok::<u32, u32>(7).slim_ok_into::<Option<_>>();
        let future_2 = future.clone();

        assert_eq!(future.await, Ok(Some(7)));
        assert_eq!(future_2.await, Ok(Some(7)));
    }

    #[tokio::test]
    async fn test_ok_into_fused_future() {
        let mut future = future::ok::<u32, u32>(7).slim_ok_into::<Option<_>>();

        assert!(!future.is_terminated());
        assert_eq!(future.by_ref().await, Ok(Some(7)));
        assert!(future.is_terminated());
    }

    #[tokio::test]
    async fn test_ok_into_is_slim() {
        let make_base_future = || crate::future::ok::<u32, u32>(2);
        let base_future = make_base_future();
        let future_1 = make_base_future().slim_ok_into::<Option<_>>();
        let future_2 = make_base_future().ok_into::<Option<_>>();

        assert_eq!(mem::size_of_val(&base_future), mem::size_of_val(&future_1));
        assert!(mem::size_of_val(&future_1) < mem::size_of_val(&future_2));
        assert_eq!(base_future.await, Ok(2));
        assert_eq!(future_1.await, Ok(Some(2)));
        assert_eq!(future_2.await, Ok(Some(2)));
    }
}
