use crate::future::map_err::MapErr;
use crate::support::fns::IntoFn;
use crate::support::TryFuture;
use futures_core::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    pub struct ErrInto<Fut, U>
    where
        Fut: TryFuture,
    {
        #[pin]
        inner: MapErr<Fut, IntoFn<Fut::Error, U>>,
    }
}

impl<Fut, U> ErrInto<Fut, U>
where
    Fut: TryFuture,
{
    pub(crate) fn new(fut: Fut) -> Self {
        Self {
            inner: MapErr::new(fut, IntoFn::default()),
        }
    }
}

impl<Fut, U> Clone for ErrInto<Fut, U>
where
    Fut: TryFuture + Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Fut, U, T, E> Future for ErrInto<Fut, U>
where
    Fut: Future<Output = Result<T, E>>,
    E: Into<U>,
{
    type Output = Result<T, U>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, E, T, U> FusedFuture for ErrInto<Fut, U>
where
    Fut: FusedFuture<Output = Result<T, E>>,
    E: Into<U>,
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
        let make_base_future = || crate::future::err::<u32, u32>(2);
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
