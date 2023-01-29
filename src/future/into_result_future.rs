use crate::future::map::Map;
use crate::support::fns::TryFromOutputFn;
use crate::support::Try;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    pub struct IntoTryFuture<Fut, T> {
        #[pin]
        inner: Map<Fut, TryFromOutputFn<T>>,
    }
}

impl<Fut, T> IntoTryFuture<Fut, T> {
    pub(crate) fn new(fut: Fut) -> Self {
        Self {
            inner: Map::new(fut, TryFromOutputFn::default()),
        }
    }
}

impl<Fut, T> Clone for IntoTryFuture<Fut, T>
where
    Fut: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Fut, T> Future for IntoTryFuture<Fut, T>
where
    Fut: Future,
    T: Try<Output = Fut::Output>,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, T> FusedFuture for IntoTryFuture<Fut, T>
where
    Fut: FusedFuture,
    T: Try<Output = Fut::Output>,
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
    use futures_core::FusedFuture;
    use futures_util::{future, FutureExt as _};
    use std::mem;
    use std::string::String;

    #[tokio::test]
    async fn test_into_try_future() {
        let future: IntoTryFuture<_, Result<_, String>> =
            future::ready(7_u32).slim_into_try_future::<Result<_, String>>();

        let result: Result<u32, String> = future.await;

        assert_eq!(result, Ok(7));
    }

    #[tokio::test]
    async fn test_never_error() {
        let future: IntoTryFuture<_, Result<_, Never>> = future::ready(7).slim_never_error();
        let result: Result<u32, Never> = future.await;

        assert_eq!(result, Ok(7));
    }

    #[tokio::test]
    async fn test_unit_error() {
        let future: IntoTryFuture<_, Result<_, ()>> = future::ready(7).slim_unit_error();
        let result: Result<u32, ()> = future.await;

        assert_eq!(result, Ok(7));
    }

    #[tokio::test]
    async fn test_into_try_future_clone() {
        let future: IntoTryFuture<_, Result<_, String>> = future::ready(7).slim_into_try_future::<Result<_, String>>();
        let future_2 = future.clone();

        assert_eq!(future.await, Ok(7));
        assert_eq!(future_2.await, Ok(7));
    }

    #[tokio::test]
    async fn test_into_try_future_fused_future() {
        let mut future: IntoTryFuture<_, Result<_, String>> =
            future::ready(7).slim_into_try_future::<Result<_, String>>();

        assert!(!future.is_terminated());
        assert_eq!(future.by_ref().await, Ok(7));
        assert!(future.is_terminated());
    }

    #[tokio::test]
    async fn test_into_try_future_is_slim() {
        let make_base_future = || crate::future::ready_by_copy::<u32>(2);
        let base_future = make_base_future();
        let future_1 = make_base_future().slim_unit_error();
        let future_2 = make_base_future().unit_error();

        assert_eq!(mem::size_of_val(&base_future), mem::size_of_val(&future_1));
        assert!(mem::size_of_val(&future_1) < mem::size_of_val(&future_2));
        assert_eq!(base_future.await, 2);
        assert_eq!(future_1.await, Ok(2));
        assert_eq!(future_2.await, Ok(2));
    }
}
