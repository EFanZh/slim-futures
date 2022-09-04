use crate::future::map::Map;
use crate::future::or_else_async::OrElseAsync;
use crate::support::fns::ErrFn;
use crate::support::{FnMut1, TryFuture};
use futures_core::FusedFuture;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

struct MapErrAsyncFn<F, T> {
    inner: F,
    _phantom: PhantomData<fn() -> T>,
}

impl<F, T> Clone for MapErrAsyncFn<F, T>
where
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<E, F, T> FnMut1<E> for MapErrAsyncFn<F, T>
where
    F: FnMut1<E>,
    F::Output: Future,
{
    type Output = Map<F::Output, ErrFn<T, <F::Output as Future>::Output>>;

    fn call_mut(&mut self, arg: E) -> Self::Output {
        Map::new(self.inner.call_mut(arg), ErrFn::default())
    }
}

pin_project_lite::pin_project! {
    pub struct MapErrAsync<Fut, F>
    where
        Fut: TryFuture,
        F: FnMut1<Fut::Error>,
        F::Output: Future,
    {
        #[pin]
        inner: OrElseAsync<Fut, MapErrAsyncFn<F, Fut::Ok>>,
    }
}

impl<Fut, F, T, E> MapErrAsync<Fut, F>
where
    Fut: Future<Output = Result<T, E>>,
    F: FnMut1<E>,
    F::Output: Future,
{
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self {
            inner: OrElseAsync::new(
                fut,
                MapErrAsyncFn {
                    inner: f,
                    _phantom: PhantomData,
                },
            ),
        }
    }
}

impl<Fut, F, T, E> Clone for MapErrAsync<Fut, F>
where
    Fut: Clone + Future<Output = Result<T, E>>,
    F: Clone + FnMut1<E>,
    F::Output: Clone + Future,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Fut, F, T, E> Future for MapErrAsync<Fut, F>
where
    Fut: Future<Output = Result<T, E>>,
    F: FnMut1<E>,
    F::Output: Future,
{
    type Output = Result<T, <F::Output as Future>::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, F, T, E> FusedFuture for MapErrAsync<Fut, F>
where
    Fut: FusedFuture<Output = Result<T, E>>,
    F: FnMut1<E>,
    F::Output: FusedFuture,
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
    async fn test_map_err_async() {
        assert_eq!(
            future::ready(Ok::<u32, u32>(2))
                .slim_map_err_async(|value| future::ready(value + 3))
                .await,
            Ok(2)
        );

        assert_eq!(
            future::ready(Err::<u32, u32>(2))
                .slim_map_err_async(|value| future::ready(value + 3))
                .await,
            Err(5)
        );
    }

    #[tokio::test]
    async fn test_map_err_async_clone() {
        let future = future::ready(Err::<u32, u32>(2)).slim_map_err_async(|value| future::ready(value + 3));
        let future_2 = future.clone();

        assert_eq!(future.await, Err(5));
        assert_eq!(future_2.await, Err(5));
    }

    #[tokio::test]
    async fn test_map_err_async_fused_future() {
        let mut future = future::ready(Err::<u32, u32>(2)).slim_map_err_async(|value| future::ready(value + 3));

        assert!(!future.is_terminated());
        assert_eq!((&mut future).await, Err(5));
        assert!(future.is_terminated());
    }
}