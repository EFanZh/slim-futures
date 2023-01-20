use crate::future::map::Map;
use crate::future::or_else_async::OrElseAsync;
use crate::support::fns::ErrFn;
use crate::support::ResultFuture;
use core::future::{Future, IntoFuture};
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::FnMut;
use futures_core::FusedFuture;

struct MapErrAsyncFn<F, T> {
    inner: F,
    phantom: PhantomData<fn() -> T>,
}

impl<F, T> Clone for MapErrAsyncFn<F, T>
where
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            phantom: self.phantom,
        }
    }
}

impl<E, F, T> FnMut<(E,)> for MapErrAsyncFn<F, T>
where
    F: FnMut<(E,)>,
    F::Output: IntoFuture,
{
    type Output = Map<<F::Output as IntoFuture>::IntoFuture, ErrFn<T>>;

    fn call_mut(&mut self, args: (E,)) -> Self::Output {
        Map::new(self.inner.call_mut(args).into_future(), ErrFn::default())
    }
}

pin_project_lite::pin_project! {
    pub struct MapErrAsync<Fut, F>
    where
        Fut: ResultFuture,
        F: FnMut<(Fut::Error,)>,
        F::Output: IntoFuture,
    {
        #[pin]
        inner: OrElseAsync<Fut, MapErrAsyncFn<F, Fut::Ok>>,
    }
}

impl<Fut, F> MapErrAsync<Fut, F>
where
    Fut: ResultFuture,
    F: FnMut<(Fut::Error,)>,
    F::Output: IntoFuture,
{
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self {
            inner: OrElseAsync::new(
                fut,
                MapErrAsyncFn {
                    inner: f,
                    phantom: PhantomData,
                },
            ),
        }
    }
}

impl<Fut, F> Clone for MapErrAsync<Fut, F>
where
    Fut: ResultFuture + Clone,
    F: FnMut<(Fut::Error,)> + Clone,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::IntoFuture: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Fut, F> Future for MapErrAsync<Fut, F>
where
    Fut: ResultFuture,
    F: FnMut<(Fut::Error,)>,
    F::Output: IntoFuture,
{
    type Output = Result<Fut::Ok, <F::Output as IntoFuture>::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, F> FusedFuture for MapErrAsync<Fut, F>
where
    Fut: ResultFuture + FusedFuture,
    F: FnMut<(Fut::Error,)>,
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
    use std::mem;
    use std::num::NonZeroU32;

    fn plus_3(value: u32) -> impl FusedFuture<Output = u32> + Clone {
        future::ready(value + 3)
    }

    #[tokio::test]
    async fn test_map_err_async() {
        assert_eq!(future::ok::<u32, _>(2).slim_map_err_async(plus_3).await, Ok(2));
        assert_eq!(future::err::<u32, _>(2).slim_map_err_async(plus_3).await, Err(5));
    }

    #[tokio::test]
    async fn test_map_err_async_clone() {
        let future = future::err::<u32, _>(2).slim_map_err_async(plus_3);
        let future_2 = future.clone();

        assert_eq!(future.await, Err(5));
        assert_eq!(future_2.await, Err(5));
    }

    #[tokio::test]
    async fn test_map_err_async_fused_future() {
        let mut future = future::err::<u32, _>(2).slim_map_err_async(plus_3);

        assert!(!future.is_terminated());
        assert_eq!(future.by_ref().await, Err(5));
        assert!(future.is_terminated());
    }

    #[tokio::test]
    async fn test_map_err_async_is_slim() {
        let make_base_future = || crate::future::err::<u32, _>(NonZeroU32::new(2).unwrap()).slim_map_err(drop);
        let base_future = make_base_future();
        let future = make_base_future().slim_map_err_async(crate::future::ready);

        assert_eq!(mem::size_of_val(&base_future), mem::size_of_val(&future));
        assert_eq!(base_future.await, Err(()));
        assert_eq!(future.await, Err(()));
    }
}
