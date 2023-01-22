use crate::future::map::Map;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::fns::ResultOkFn;
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    pub struct IntoResultFuture<Fut, E> {
        #[pin]
        inner: Map<Fut, ResultOkFn<E>>,
    }
}

impl<Fut, E> IntoResultFuture<Fut, E> {
    pub(crate) fn new(fut: Fut) -> Self {
        Self {
            inner: Map::new(fut, ResultOkFn::default()),
        }
    }
}

impl<Fut, E> Clone for IntoResultFuture<Fut, E>
where
    Fut: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Fut, E> Future for IntoResultFuture<Fut, E>
where
    Fut: Future,
{
    type Output = Result<Fut::Output, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, E> FusedFuture for IntoResultFuture<Fut, E>
where
    Fut: FusedFuture,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

#[cfg(test)]
mod tests {
    use super::IntoResultFuture;
    use crate::future::future_ext::FutureExt;
    use crate::support::Never;
    use futures_core::FusedFuture;
    use futures_util::{future, FutureExt as _};
    use std::mem;
    use std::string::String;

    #[tokio::test]
    async fn test_into_try_future() {
        let future: IntoResultFuture<_, String> = future::ready(7).slim_into_result_future::<String>();
        let result: Result<u32, String> = future.await;

        assert_eq!(result, Ok(7));
    }

    #[tokio::test]
    async fn test_never_error() {
        let future: IntoResultFuture<_, Never> = future::ready(7).slim_never_error();
        let result: Result<u32, Never> = future.await;

        assert_eq!(result, Ok(7));
    }

    #[tokio::test]
    async fn test_unit_error() {
        let future: IntoResultFuture<_, ()> = future::ready(7).slim_unit_error();
        let result: Result<u32, ()> = future.await;

        assert_eq!(result, Ok(7));
    }

    #[tokio::test]
    async fn test_into_try_future_clone() {
        let future: IntoResultFuture<_, String> = future::ready(7).slim_into_result_future::<String>();
        let future_2 = future.clone();

        assert_eq!(future.await, Ok(7));
        assert_eq!(future_2.await, Ok(7));
    }

    #[tokio::test]
    async fn test_into_try_future_fused_future() {
        let mut future: IntoResultFuture<_, String> = future::ready(7).slim_into_result_future::<String>();

        assert!(!future.is_terminated());
        assert_eq!(future.by_ref().await, Ok(7));
        assert!(future.is_terminated());
    }

    #[tokio::test]
    async fn test_into_try_future_is_slim() {
        let make_base_future = || crate::future::ready::<u32>(2);
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
