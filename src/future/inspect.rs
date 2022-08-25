use crate::future::map::Map;
use crate::support::FnMut1;
use futures_core::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Clone)]
struct InspectFn<F> {
    inner: F,
}

impl<T, F> FnMut1<T> for InspectFn<F>
where
    F: for<'a> FnMut1<&'a T, Output = ()>,
{
    type Output = T;

    fn call_mut(&mut self, arg: T) -> Self::Output {
        self.inner.call_mut(&arg);

        arg
    }
}

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
            inner: Map::new(fut, InspectFn { inner: f }),
        }
    }
}

impl<Fut, F> Future for Inspect<Fut, F>
where
    Fut: Future,
    F: FnMut(&Fut::Output),
{
    type Output = Fut::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, F> FusedFuture for Inspect<Fut, F>
where
    Fut: FusedFuture,
    F: FnMut(&Fut::Output),
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::future;
    use crate::future::future_ext::FutureExt;
    use futures_core::FusedFuture;
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
        let mut future = futures_util::future::ready(()).slim_inspect(|_| {});

        assert!(!future.is_terminated());

        (&mut future).await;

        assert!(future.is_terminated());
    }
}
