use crate::future::map::Map;
use crate::support::FnMut1;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::FusedFuture;

#[derive(Clone)]
struct MapOkFn<F> {
    inner: F,
}

impl<T, E, F> FnMut1<Result<T, E>> for MapOkFn<F>
where
    F: FnMut1<T>,
{
    type Output = Result<F::Output, E>;

    fn call_mut(&mut self, arg: Result<T, E>) -> Self::Output {
        arg.map(|value| self.inner.call_mut(value))
    }
}

pin_project_lite::pin_project! {
    #[derive(Clone)]
    pub struct MapOk<Fut, F> {
        #[pin]
        inner: Map<Fut, MapOkFn<F>>,
    }
}

impl<Fut, F> MapOk<Fut, F> {
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self {
            inner: Map::new(fut, MapOkFn { inner: f }),
        }
    }
}

impl<Fut, F, T, E> Future for MapOk<Fut, F>
where
    Fut: Future<Output = Result<T, E>>,
    F: FnMut1<T>,
{
    type Output = Result<F::Output, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, F, T, E> FusedFuture for MapOk<Fut, F>
where
    Fut: FusedFuture<Output = Result<T, E>>,
    F: FnMut1<T>,
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

    fn plus_3(value: u32) -> u32 {
        value + 3
    }

    #[tokio::test]
    async fn test_map_ok() {
        assert_eq!(future::ok::<_, u32>(2).slim_map_ok(plus_3).await, Ok(5));
        assert_eq!(future::err::<_, u32>(2).slim_map_ok(plus_3).await, Err(2));
    }

    #[tokio::test]
    async fn test_map_ok_clone() {
        let future = future::ok::<_, u32>(2).slim_map_ok(plus_3);
        let future_2 = future.clone();

        assert_eq!(future.await, Ok(5));
        assert_eq!(future_2.await, Ok(5));
    }

    #[tokio::test]
    async fn test_map_ok_fused_future() {
        let mut future = future::ok::<_, u32>(2).slim_map_ok(plus_3);

        assert!(!future.is_terminated());
        assert_eq!(future.by_ref().await, Ok(5));
        assert!(future.is_terminated());
    }

    #[tokio::test]
    async fn test_map_ok_is_slim() {
        let make_base_future = || crate::future::ok::<u32, u32>(2);
        let base_future = make_base_future();
        let future_1 = make_base_future().slim_map_ok(plus_3);
        let future_2 = make_base_future().map_ok(plus_3);

        assert_eq!(mem::size_of_val(&base_future), mem::size_of_val(&future_1));
        assert!(mem::size_of_val(&future_1) < mem::size_of_val(&future_2));
        assert_eq!(base_future.await, Ok(2));
        assert_eq!(future_1.await, Ok(5));
        assert_eq!(future_2.await, Ok(5));
    }
}
