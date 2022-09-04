use crate::future::map_err::MapErr;
use crate::future::try_flatten_err::TryFlattenErr;
use crate::support::{FnMut1, TryFuture};
use futures_core::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    pub struct OrElseAsync<Fut, F>
    where
        Fut: TryFuture,
        F: FnMut1<Fut::Error>,
    {
        #[pin]
        inner: TryFlattenErr<MapErr<Fut, F>>
    }
}

impl<Fut, F, T, E> OrElseAsync<Fut, F>
where
    Fut: Future<Output = Result<T, E>>,
    F: FnMut1<E>,
{
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self {
            inner: TryFlattenErr::new(MapErr::new(fut, f)),
        }
    }
}

impl<Fut, F, T, E> Clone for OrElseAsync<Fut, F>
where
    Fut: Clone + Future<Output = Result<T, E>>,
    F: Clone + FnMut1<E>,
    F::Output: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Fut, F, T, E, U> Future for OrElseAsync<Fut, F>
where
    Fut: Future<Output = Result<T, E>>,
    F: FnMut1<E>,
    F::Output: Future<Output = Result<T, U>>,
{
    type Output = Result<T, U>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, F, T, E, U> FusedFuture for OrElseAsync<Fut, F>
where
    Fut: FusedFuture<Output = Result<T, E>>,
    F: FnMut1<E>,
    F::Output: FusedFuture<Output = Result<T, U>>,
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
    async fn test_or_else_async() {
        assert_eq!(
            future::ok::<u32, u32>(2)
                .slim_or_else_async(|value| future::ready(Ok::<u32, u32>(value + 3)))
                .await,
            Ok(2)
        );

        assert_eq!(
            future::ok::<u32, u32>(2)
                .slim_or_else_async(|value| future::ready(Err(value + 3)))
                .await,
            Ok(2)
        );

        assert_eq!(
            future::err::<u32, u32>(2)
                .slim_or_else_async(|value| future::ready(Ok::<u32, u32>(value + 3)))
                .await,
            Ok(5)
        );

        assert_eq!(
            future::err::<u32, u32>(2)
                .slim_or_else_async(|value| future::ready(Err(value + 3)))
                .await,
            Err(5)
        );
    }

    #[tokio::test]
    async fn test_or_else_async_clone() {
        let future = future::err::<u32, u32>(2).slim_or_else_async(|value| future::ready(Err(value + 3)));
        let future_2 = future.clone();

        assert_eq!(future.await, Err(5));
        assert_eq!(future_2.await, Err(5));
    }

    #[tokio::test]
    async fn test_or_else_async_fused_future() {
        let mut future = future::err::<u32, u32>(2).slim_or_else_async(|value| future::ready(Err(value + 3)));

        assert!(!future.is_terminated());
        assert_eq!((&mut future).await, Err(5));
        assert!(future.is_terminated());
    }
}
