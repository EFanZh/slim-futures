use crate::future::inspect::Inspect;
use crate::support::ResultFuture;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::FnMut;
use futures_core::FusedFuture;

#[derive(Clone)]
struct InspectOkFn<F> {
    inner: F,
}

impl<'a, T, E, F> FnMut<(&'a Result<T, E>,)> for InspectOkFn<F>
where
    F: FnMut<(&'a T,), Output = ()>,
{
    type Output = ();

    fn call_mut(&mut self, args: (&'a Result<T, E>,)) -> Self::Output {
        if let Ok(value) = args.0 {
            self.inner.call_mut((value,));
        }
    }
}

pin_project_lite::pin_project! {
    #[derive(Clone)]
    pub struct InspectOk<Fut, F> {
        #[pin]
        inner: Inspect<Fut, InspectOkFn<F>>,
    }
}

impl<Fut, F> InspectOk<Fut, F> {
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self {
            inner: Inspect::new(fut, InspectOkFn { inner: f }),
        }
    }
}

impl<Fut, F> Future for InspectOk<Fut, F>
where
    Fut: ResultFuture,
    F: for<'a> FnMut<(&'a Fut::Ok,), Output = ()>,
{
    type Output = Fut::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, F> FusedFuture for InspectOk<Fut, F>
where
    Fut: ResultFuture + FusedFuture,
    F: for<'a> FnMut<(&'a Fut::Ok,), Output = ()>,
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
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_inspect_ok() {
        let state = AtomicUsize::new(1);

        let f = |&value: &usize| {
            state
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |state| Some(state * value))
                .unwrap();
        };

        let future_1 = future::ok::<usize, usize>(2).slim_inspect_ok(f);
        let future_2 = future::err::<usize, usize>(3).slim_inspect_ok(f);

        assert_eq!(state.load(Ordering::Relaxed), 1);
        assert_eq!(future_1.await, Ok(2));
        assert_eq!(state.load(Ordering::Relaxed), 2);
        assert_eq!(future_2.await, Err(3));
        assert_eq!(state.load(Ordering::Relaxed), 2);
    }

    #[tokio::test]
    async fn test_inspect_ok_clone() {
        let state = AtomicUsize::new(2);

        let future = future::ok::<usize, usize>(3).slim_inspect_ok(|&value| {
            state.fetch_add(value, Ordering::Relaxed);
        });

        let future_2 = future.clone();

        assert_eq!(state.load(Ordering::Relaxed), 2);
        assert_eq!(future.await, Ok(3));
        assert_eq!(state.load(Ordering::Relaxed), 5);
        assert_eq!(future_2.await, Ok(3));
        assert_eq!(state.load(Ordering::Relaxed), 8);
    }

    #[tokio::test]
    async fn test_inspect_ok_fused_future() {
        let mut future = future::ok::<(), ()>(()).slim_inspect_ok(|_| {});

        assert!(!future.is_terminated());
        assert_eq!(future.by_ref().await, Ok(()));
        assert!(future.is_terminated());
    }

    #[tokio::test]
    async fn test_inspect_ok_is_slim() {
        let make_base_future = || crate::future::ok::<u32, u32>(2);
        let base_future = make_base_future();
        let future_1 = make_base_future().slim_inspect_ok(|_| {});
        let future_2 = make_base_future().inspect_ok(|_| {});

        assert_eq!(mem::size_of_val(&base_future), mem::size_of_val(&future_1));
        assert!(mem::size_of_val(&future_1) < mem::size_of_val(&future_2));
        assert_eq!(base_future.await, Ok(2));
        assert_eq!(future_1.await, Ok(2));
        assert_eq!(future_2.await, Ok(2));
    }
}
