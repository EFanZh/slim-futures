use crate::future::map_err::MapErr;
use crate::support::fns::IntoFn;
use crate::support::TryFuture;
use futures_core::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    #[derive(Clone)]
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
    use futures_util::future;

    #[tokio::test]
    async fn test_err_into() {
        assert_eq!(
            future::ready(Ok::<u32, u32>(7)).slim_err_into::<Option<_>>().await,
            Ok(7),
        );

        assert_eq!(
            future::ready(Err::<u32, u32>(7)).slim_err_into::<Option<_>>().await,
            Err(Some(7)),
        );
    }

    #[tokio::test]
    async fn test_err_into_clone() {
        let future = future::ready(Err::<u32, u32>(7)).slim_err_into::<Option<_>>();
        let future_2 = future.clone();

        assert_eq!(future.await, Err(Some(7)));
        assert_eq!(future_2.await, Err(Some(7)));
    }

    #[tokio::test]
    async fn test_err_into_fused_future() {
        let mut future = future::ready(Err::<u32, u32>(7)).slim_err_into::<Option<_>>();

        assert!(!future.is_terminated());
        assert_eq!((&mut future).await, Err(Some(7)));
        assert!(future.is_terminated());
    }
}
