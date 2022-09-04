use crate::future::map::Map;
use crate::support::FnMut1;
use futures_core::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

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
    use futures_util::future;

    #[tokio::test]
    async fn test_map_ok() {
        assert_eq!(future::ok::<u32, u32>(2).slim_map_ok(|value| value + 3).await, Ok(5));

        assert_eq!(future::err::<u32, u32>(2).slim_map_ok(|value| value + 3).await, Err(2));
    }

    #[tokio::test]
    async fn test_map_ok_clone() {
        let future = future::ok::<u32, u32>(2).slim_map_ok(|value| value + 3);
        let future_2 = future.clone();

        assert_eq!(future.await, Ok(5));
        assert_eq!(future_2.await, Ok(5));
    }

    #[tokio::test]
    async fn test_map_ok_fused_future() {
        let mut future = future::ok::<u32, u32>(2).slim_map_ok(|value| value + 3);

        assert!(!future.is_terminated());
        assert_eq!((&mut future).await, Ok(5));
        assert!(future.is_terminated());
    }
}
