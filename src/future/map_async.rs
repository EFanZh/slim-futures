use crate::future::flatten::Flatten;
use crate::future::map::Map;
use crate::support::FnMut1;
use futures_core::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    pub struct MapAsync<Fut, F>
    where
        Fut: Future,
        F: FnMut1<Fut::Output>,
    {
        #[pin]
        inner: Flatten<Map<Fut, F>>
    }
}

impl<Fut, F> MapAsync<Fut, F>
where
    Fut: Future,
    F: FnMut1<Fut::Output>,
{
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self {
            inner: Flatten::new(Map::new(fut, f)),
        }
    }
}

impl<Fut, F> Clone for MapAsync<Fut, F>
where
    Fut: Future + Clone,
    F: FnMut1<Fut::Output> + Clone,
    F::Output: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Fut, F> Future for MapAsync<Fut, F>
where
    Fut: Future,
    F: FnMut1<Fut::Output>,
    F::Output: Future,
{
    type Output = <F::Output as Future>::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, F> FusedFuture for MapAsync<Fut, F>
where
    Fut: FusedFuture,
    F: FnMut1<Fut::Output>,
    F::Output: FusedFuture,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::future::future_ext::FutureExt;
    use futures_core::FusedFuture;
    use futures_util::{future, FutureExt as _};
    use std::mem;
    use std::num::NonZeroU32;

    #[tokio::test]
    async fn test_map_async() {
        let future = future::ready(7).slim_map_async(|value| future::lazy(move |_| value + 2));

        assert_eq!(future.await, 9);
    }

    #[tokio::test]
    async fn test_map_async_clone() {
        let future = future::ready(7).slim_map_async(|value| crate::future::lazy(move |_| value + 2));
        let future_2 = future.clone();

        assert_eq!(future.await, 9);
        assert_eq!(future_2.await, 9);
    }

    #[tokio::test]
    async fn test_map_async_fused_future() {
        let mut future = future::ready(7).slim_map_async(|value| future::lazy(move |_| value + 2));

        assert!(!future.is_terminated());
        assert_eq!(future.by_ref().await, 9);
        assert!(future.is_terminated());
    }

    #[tokio::test]
    async fn test_map_async_is_slim() {
        let make_base_future = || crate::future::ready(NonZeroU32::new(2).unwrap()).slim_map(drop);
        let future_1 = make_base_future().slim_map_async(crate::future::ready);
        let future_2 = make_base_future().then(crate::future::ready);

        assert!(mem::size_of_val(&future_1) < mem::size_of_val(&future_2));
        assert!(matches!(future_1.await, ()));
        assert!(matches!(future_2.await, ()));
    }
}
