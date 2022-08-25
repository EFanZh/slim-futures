use crate::support::FnMut1;
use futures_core::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    pub struct Map<Fut, F> {
        #[pin]
        fut: Fut,
        f: F,
    }
}

impl<Fut, F> Map<Fut, F> {
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self { fut, f }
    }
}

// Manual implement `Clone` to avoid inlining.
impl<Fut, F> Clone for Map<Fut, F>
where
    Fut: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            fut: self.fut.clone(),
            f: self.f.clone(),
        }
    }
}

impl<Fut, F> Future for Map<Fut, F>
where
    Fut: Future,
    F: FnMut1<Fut::Output>,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();

        this.fut.poll(cx).map(|value| this.f.call_mut(value))
    }
}

impl<Fut, F> FusedFuture for Map<Fut, F>
where
    Fut: FusedFuture,
    F: FnMut1<Fut::Output>,
{
    fn is_terminated(&self) -> bool {
        self.fut.is_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::future::future_ext::FutureExt;
    use crate::future::ready;
    use futures_core::FusedFuture;

    #[tokio::test]
    async fn test_map() {
        assert_eq!(ready(2).slim_map(|value| value + 3).await, 5);
    }

    #[tokio::test]
    async fn test_map_clone() {
        let future = ready(2).slim_map(|value| value + 3);
        let future_2 = future.clone();

        assert_eq!(future.await, 5);
        assert_eq!(future_2.await, 5);
    }

    #[tokio::test]
    async fn test_map_fused_future() {
        let mut future = futures_util::future::ready(2).slim_map(|value| value + 3);

        assert!(!future.is_terminated());
        assert_eq!((&mut future).await, 5);
        assert!(future.is_terminated());
    }
}
