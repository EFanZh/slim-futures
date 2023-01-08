use crate::support::FnMut1;
use core::future::Future;
use core::pin::Pin;
use core::task::{self, Context, Poll};
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    #[derive(Clone)]
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

impl<Fut, F> Future for Map<Fut, F>
where
    Fut: Future,
    F: FnMut1<Fut::Output>,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();

        Poll::Ready(this.f.call_mut(task::ready!(this.fut.poll(cx))))
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
    use futures_core::FusedFuture;
    use futures_util::{future, FutureExt as _};
    use std::mem;

    fn plus_3(value: u32) -> u32 {
        value + 3
    }

    #[tokio::test]
    async fn test_map() {
        assert_eq!(future::ready(2).slim_map(plus_3).await, 5);
    }

    #[tokio::test]
    async fn test_map_clone() {
        let future = future::ready(2).slim_map(plus_3);
        let future_2 = future.clone();

        assert_eq!(future.await, 5);
        assert_eq!(future_2.await, 5);
    }

    #[tokio::test]
    async fn test_map_fused_future() {
        let mut future = future::ready(2).slim_map(plus_3);

        assert!(!future.is_terminated());
        assert_eq!(future.by_ref().await, 5);
        assert!(future.is_terminated());
    }

    #[tokio::test]
    async fn test_map_is_slim() {
        let make_base_future = || crate::future::ready(2);
        let base_future = make_base_future();
        let future_1 = make_base_future().slim_map(plus_3);
        let future_2 = make_base_future().map(plus_3);

        assert_eq!(mem::size_of_val(&base_future), mem::size_of_val(&future_1));
        assert!(mem::size_of_val(&future_1) < mem::size_of_val(&future_2));
        assert_eq!(base_future.await, 2);
        assert_eq!(future_1.await, 5);
        assert_eq!(future_2.await, 5);
    }
}
