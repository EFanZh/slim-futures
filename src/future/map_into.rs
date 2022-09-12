use crate::future::map::Map;
use crate::support::fns::IntoFn;
use futures_core::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    pub struct MapInto<Fut, T>
    where
        Fut: Future,
    {
        #[pin]
        inner: Map<Fut, IntoFn<Fut::Output, T>>,
    }
}

impl<Fut, T> Clone for MapInto<Fut, T>
where
    Fut: Clone + Future,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Fut, T> MapInto<Fut, T>
where
    Fut: Future,
{
    pub(crate) fn new(fut: Fut) -> Self {
        Self {
            inner: Map::new(fut, IntoFn::default()),
        }
    }
}

impl<Fut, T> Future for MapInto<Fut, T>
where
    Fut: Future,
    Fut::Output: Into<T>,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, T> FusedFuture for MapInto<Fut, T>
where
    Fut: FusedFuture,
    Fut::Output: Into<T>,
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
    async fn test_map_into() {
        let future = future::ready(7_u32).slim_map_into::<Option<_>>();

        assert_eq!(future.await, Some(7));
    }

    #[tokio::test]
    async fn test_map_into_clone() {
        let future = future::ready(7_u32).slim_map_into::<Option<_>>();
        let future_2 = future.clone();

        assert_eq!(future.await, Some(7));
        assert_eq!(future_2.await, Some(7));
    }

    #[tokio::test]
    async fn test_map_into_fused_future() {
        let mut future = future::ready(7_u32).slim_map_into::<Option<_>>();

        assert!(!future.is_terminated());
        assert_eq!(future.by_ref().await, Some(7));
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
