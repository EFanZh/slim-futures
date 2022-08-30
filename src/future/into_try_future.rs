use crate::future::map::Map;
use crate::support::fns::OkFn;
use futures_core::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    pub struct IntoTryFuture<Fut, E>
    where
        Fut: Future,
    {
        #[pin]
        inner: Map<Fut, OkFn<Fut::Output, E>>,
    }
}

impl<Fut, E> IntoTryFuture<Fut, E>
where
    Fut: Future,
{
    pub(crate) fn new(fut: Fut) -> Self {
        Self {
            inner: Map::new(fut, OkFn::default()),
        }
    }
}

impl<Fut, E> Clone for IntoTryFuture<Fut, E>
where
    Fut: Clone + Future,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Fut, E> Future for IntoTryFuture<Fut, E>
where
    Fut: Future,
{
    type Output = Result<Fut::Output, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, E> FusedFuture for IntoTryFuture<Fut, E>
where
    Fut: FusedFuture,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

#[cfg(test)]
mod tests {
    use super::IntoTryFuture;
    use crate::future::future_ext::FutureExt;
    use crate::support::Never;
    use crate::test_utilities;
    use futures_core::FusedFuture;
    use futures_util::{future, FutureExt as _};
    use std::mem;

    #[tokio::test]
    async fn test_into_try_future() {
        let future: IntoTryFuture<_, String> = future::ready(7).slim_into_try_future::<String>();
        let result: Result<u32, String> = future.await;

        assert_eq!(result, Ok(7));
    }

    #[tokio::test]
    async fn test_never_error() {
        let future: IntoTryFuture<_, Never> = future::ready(7).slim_never_error();
        let result: Result<u32, Never> = future.await;

        assert_eq!(result, Ok(7));
    }

    #[tokio::test]
    async fn test_unit_error() {
        let future: IntoTryFuture<_, ()> = future::ready(7).slim_unit_error();
        let result: Result<u32, ()> = future.await;

        assert_eq!(result, Ok(7));
    }

    #[tokio::test]
    async fn test_into_try_future_clone() {
        let future: IntoTryFuture<_, String> = future::ready(7).slim_into_try_future::<String>();
        let future_2 = future.clone();

        assert_eq!(future.await, Ok(7));
        assert_eq!(future_2.await, Ok(7));
    }

    #[tokio::test]
    async fn test_into_try_future_fused_future() {
        let mut future: IntoTryFuture<_, String> = future::ready(7).slim_into_try_future::<String>();

        assert!(!future.is_terminated());
        assert_eq!((&mut future).await, Ok(7));
        assert!(future.is_terminated());
    }

    #[test]
    fn test_into_try_future_is_slim() {
        let make_future = test_utilities::full_bytes_future;
        let future = make_future().slim_unit_error();
        let other = make_future().unit_error();

        assert!(mem::size_of_val(&future) < mem::size_of_val(&other));
    }
}
