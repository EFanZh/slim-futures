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
    use futures_util::future;

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
        assert_eq!((&mut future).await, 9);
        assert!(future.is_terminated());
    }
}
