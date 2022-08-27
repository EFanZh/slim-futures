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
    use futures_util::future;

    #[tokio::test]
    async fn test_and_then_async() {
        assert_eq!(
            future::ready(Ok::<u32, u32>(2))
                .slim_and_then_async(|value| future::ready(Ok(value + 3)))
                .await,
            Ok(5)
        );

        assert_eq!(
            future::ready(Ok::<u32, u32>(2))
                .slim_and_then_async(|value| future::ready(Err::<u32, u32>(value + 3)))
                .await,
            Err(5)
        );

        assert_eq!(
            future::ready(Err::<u32, u32>(2))
                .slim_and_then_async(|value| future::ready(Ok(value + 3)))
                .await,
            Err(2)
        );

        assert_eq!(
            future::ready(Err::<u32, u32>(2))
                .slim_and_then_async(|value| future::ready(Err::<u32, u32>(value + 3)))
                .await,
            Err(2)
        );
    }

    #[tokio::test]
    async fn test_and_then_async_clone() {
        let future = future::ready(Ok::<u32, u32>(2)).slim_and_then_async(|value| future::ready(Ok(value + 3)));
        let future_2 = future.clone();

        assert_eq!(future.await, Ok(5));
        assert_eq!(future_2.await, Ok(5));
    }

    #[tokio::test]
    async fn test_and_then_async_fused_future() {
        let mut future = future::ready(Ok::<u32, u32>(2)).slim_and_then_async(|value| future::ready(Ok(value + 3)));

        assert!(!future.is_terminated());
        assert_eq!((&mut future).await, Ok(5));
        assert!(future.is_terminated());
    }
}
