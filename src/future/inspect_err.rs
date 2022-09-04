use crate::future::inspect::Inspect;
use crate::support::FnMut1;
use futures_core::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Clone)]
struct InspectErrFn<F> {
    inner: F,
}

impl<'a, T, E, F> FnMut1<&'a Result<T, E>> for InspectErrFn<F>
where
    F: FnMut1<&'a E, Output = ()>,
{
    type Output = ();

    fn call_mut(&mut self, arg: &'a Result<T, E>) -> Self::Output {
        if let Err(error) = arg {
            self.inner.call_mut(error);
        }
    }
}

pin_project_lite::pin_project! {
    #[derive(Clone)]
    pub struct InspectErr<Fut, F> {
        #[pin]
        inner: Inspect<Fut, InspectErrFn<F>>,
    }
}

impl<Fut, F> InspectErr<Fut, F> {
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self {
            inner: Inspect::new(fut, InspectErrFn { inner: f }),
        }
    }
}

impl<Fut, F, T, E> Future for InspectErr<Fut, F>
where
    Fut: Future<Output = Result<T, E>>,
    F: for<'a> FnMut1<&'a E, Output = ()>,
{
    type Output = Fut::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, F, T, E> FusedFuture for InspectErr<Fut, F>
where
    Fut: FusedFuture<Output = Result<T, E>>,
    F: FnMut(&E),
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
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_inspect_err() {
        let state = AtomicUsize::new(1);

        let f = |&value: &usize| {
            state
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |state| Some(state * value))
                .unwrap();
        };

        let future_1 = future::ready(Ok::<usize, usize>(2)).slim_inspect_err(f);
        let future_2 = future::ready(Err::<usize, usize>(3)).slim_inspect_err(f);

        assert_eq!(state.load(Ordering::Relaxed), 1);
        assert_eq!(future_1.await, Ok(2));
        assert_eq!(state.load(Ordering::Relaxed), 1);
        assert_eq!(future_2.await, Err(3));
        assert_eq!(state.load(Ordering::Relaxed), 3);
    }

    #[tokio::test]
    async fn test_inspect_err_clone() {
        let state = AtomicUsize::new(2);

        let future = future::ready(Err::<usize, usize>(3)).slim_inspect_err(|&value| {
            state.fetch_add(value, Ordering::Relaxed);
        });

        let future_2 = future.clone();

        assert_eq!(state.load(Ordering::Relaxed), 2);
        assert_eq!(future.await, Err(3));
        assert_eq!(state.load(Ordering::Relaxed), 5);
        assert_eq!(future_2.await, Err(3));
        assert_eq!(state.load(Ordering::Relaxed), 8);
    }

    #[tokio::test]
    async fn test_inspect_err_fused_future() {
        let mut future = future::ready(Err::<(), ()>(())).slim_inspect_err(|_| {});

        assert!(!future.is_terminated());
        assert_eq!((&mut future).await, Err(()));
        assert!(future.is_terminated());
    }
}