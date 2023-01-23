use crate::future::map::Map;
use crate::support::fns::InspectFn;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::FnMut;
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    #[derive(Clone)]
    pub struct Inspect<Fut, F> {
        #[pin]
        inner: Map<Fut, InspectFn<F>>,
    }
}

impl<Fut, F> Inspect<Fut, F> {
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self {
            inner: Map::new(fut, InspectFn::new(f)),
        }
    }
}

impl<Fut, F> Future for Inspect<Fut, F>
where
    Fut: Future,
    F: for<'a> FnMut<(&'a Fut::Output,), Output = ()>,
{
    type Output = Fut::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, F> FusedFuture for Inspect<Fut, F>
where
    Fut: FusedFuture,
    F: for<'a> FnMut<(&'a Fut::Output,), Output = ()>,
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
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_inspect() {
        let state = AtomicUsize::new(2);

        let future = future::ready(3).slim_inspect(|&value| {
            state.fetch_add(value, Ordering::Relaxed);
        });

        assert_eq!(state.load(Ordering::Relaxed), 2);
        assert_eq!(future.await, 3);
        assert_eq!(state.load(Ordering::Relaxed), 5);
    }

    #[tokio::test]
    async fn test_inspect_clone() {
        let state = AtomicUsize::new(2);

        let future = future::ready(3).slim_inspect(|&value| {
            state.fetch_add(value, Ordering::Relaxed);
        });

        let future_2 = future.clone();

        assert_eq!(state.load(Ordering::Relaxed), 2);
        assert_eq!(future.await, 3);
        assert_eq!(state.load(Ordering::Relaxed), 5);
        assert_eq!(future_2.await, 3);
        assert_eq!(state.load(Ordering::Relaxed), 8);
    }

    #[tokio::test]
    async fn test_inspect_fused_future() {
        let mut future = future::ready(()).slim_inspect(|_| {});

        assert!(!future.is_terminated());

        future.by_ref().await;

        assert!(future.is_terminated());
    }

    #[tokio::test]
    async fn test_inspect_is_slim() {
        let make_base_future = || crate::future::ready_by_copy::<u32>(2);
        let base_future = make_base_future();
        let future_1 = make_base_future().slim_inspect(|_| {});
        let future_2 = make_base_future().inspect(|_| {});

        assert_eq!(mem::size_of_val(&base_future), mem::size_of_val(&future_1));
        assert!(mem::size_of_val(&future_1) < mem::size_of_val(&future_2));
        assert_eq!(base_future.await, 2);
        assert_eq!(future_1.await, 2);
        assert_eq!(future_2.await, 2);
    }
}
