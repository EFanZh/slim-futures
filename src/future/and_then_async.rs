use crate::future::map_ok::MapOk;
use crate::future::try_flatten::TryFlatten;
use crate::support::{FnMut1, TryFuture};
use futures_core::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    pub struct AndThenAsync<Fut, F>
    where
        Fut: TryFuture,
        F: FnMut1<Fut::Ok>,
    {
        #[pin]
        inner: TryFlatten<MapOk<Fut, F>>
    }
}

impl<Fut, F, T, E> AndThenAsync<Fut, F>
where
    Fut: Future<Output = Result<T, E>>,
    F: FnMut1<T>,
{
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self {
            inner: TryFlatten::new(MapOk::new(fut, f)),
        }
    }
}

impl<Fut, F, T, E> Clone for AndThenAsync<Fut, F>
where
    Fut: Clone + Future<Output = Result<T, E>>,
    F: Clone + FnMut1<T>,
    F::Output: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Fut, F, T, E, U> Future for AndThenAsync<Fut, F>
where
    Fut: Future<Output = Result<T, E>>,
    F: FnMut1<T>,
    F::Output: Future<Output = Result<U, E>>,
{
    type Output = Result<U, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, F, T, E, U> FusedFuture for AndThenAsync<Fut, F>
where
    Fut: FusedFuture<Output = Result<T, E>>,
    F: FnMut1<T>,
    F::Output: FusedFuture<Output = Result<U, E>>,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::future::future_ext::FutureExt;
    use futures_core::FusedFuture;
    use futures_util::future::{self, Ready, TryFutureExt};
    use std::mem;

    fn ok_plus_3(value: u32) -> Ready<Result<u32, u32>> {
        future::ok(value + 3)
    }

    fn err_plus_3(value: u32) -> Ready<Result<u32, u32>> {
        future::err(value + 3)
    }

    #[tokio::test]
    async fn test_and_then_async() {
        assert_eq!(future::ready(Ok(2)).slim_and_then_async(ok_plus_3).await, Ok(5));
        assert_eq!(future::ready(Ok(2)).slim_and_then_async(err_plus_3).await, Err(5));
        assert_eq!(future::ready(Err(2)).slim_and_then_async(ok_plus_3).await, Err(2));
        assert_eq!(future::ready(Err(2)).slim_and_then_async(err_plus_3).await, Err(2));
    }

    #[tokio::test]
    async fn test_and_then_async_clone() {
        let future = future::ready(Ok(2)).slim_and_then_async(ok_plus_3);
        let future_2 = future.clone();

        assert_eq!(future.await, Ok(5));
        assert_eq!(future_2.await, Ok(5));
    }

    #[tokio::test]
    async fn test_and_then_async_fused_future() {
        let mut future = future::ready(Ok(2)).slim_and_then_async(ok_plus_3);

        assert!(!future.is_terminated());
        assert_eq!((&mut future).await, Ok(5));
        assert!(future.is_terminated());
    }

    #[tokio::test]
    async fn test_and_then_async_is_slim() {
        let make_base_future = || crate::future::ok::<u32, u32>(2);
        let future_1 = make_base_future().slim_and_then_async(crate::future::ok);
        let future_2 = make_base_future().and_then(crate::future::ok);

        assert!(mem::size_of_val(&future_1) < mem::size_of_val(&future_2));
        assert_eq!(future_1.await, Ok(2));
        assert_eq!(future_2.await, Ok(2));
    }
}
