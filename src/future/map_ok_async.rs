use crate::future::and_then_async::AndThenAsync;
use crate::future::map::Map;
use crate::support::ResultFuture;
use core::future::{Future, IntoFuture};
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::fns::ResultOkFn;
use fn_traits::FnMut;
use futures_core::FusedFuture;

struct MapOkAsyncFn<F, E> {
    inner: F,
    phantom: PhantomData<fn() -> E>,
}

impl<F, E> Clone for MapOkAsyncFn<F, E>
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

impl<T, F, E> FnMut<(T,)> for MapOkAsyncFn<F, E>
where
    F: FnMut<(T,)>,
    F::Output: IntoFuture,
{
    type Output = Map<<F::Output as IntoFuture>::IntoFuture, ResultOkFn<E>>;

    fn call_mut(&mut self, args: (T,)) -> Self::Output {
        Map::new(self.inner.call_mut(args).into_future(), ResultOkFn::default())
    }
}

pin_project_lite::pin_project! {
    pub struct MapOkAsync<Fut, F>
    where
        Fut: ResultFuture,
        F: FnMut<(Fut::Ok,)>,
        F::Output: IntoFuture,
    {
        #[pin]
        inner: AndThenAsync<Fut, MapOkAsyncFn<F, Fut::Error>>,
    }
}

impl<Fut, F> MapOkAsync<Fut, F>
where
    Fut: ResultFuture,
    F: FnMut<(Fut::Ok,)>,
    F::Output: IntoFuture,
{
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self {
            inner: AndThenAsync::new(
                fut,
                MapOkAsyncFn {
                    inner: f,
                    phantom: PhantomData,
                },
            ),
        }
    }
}

impl<Fut, F> Clone for MapOkAsync<Fut, F>
where
    Fut: ResultFuture + Clone,
    F: FnMut<(Fut::Ok,)> + Clone,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::IntoFuture: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Fut, F> Future for MapOkAsync<Fut, F>
where
    Fut: ResultFuture,
    F: FnMut<(Fut::Ok,)>,
    F::Output: IntoFuture,
{
    type Output = Result<<F::Output as IntoFuture>::Output, Fut::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, F> FusedFuture for MapOkAsync<Fut, F>
where
    Fut: ResultFuture + FusedFuture,
    F: FnMut<(Fut::Ok,)>,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::IntoFuture: FusedFuture,
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
    async fn test_map_ok_async() {
        assert_eq!(future::ok::<_, u32>(2).slim_map_ok_async(plus_3).await, Ok(5));
        assert_eq!(future::err::<_, u32>(2).slim_map_ok_async(plus_3).await, Err(2));
    }

    #[tokio::test]
    async fn test_map_ok_async_clone() {
        let future = future::ok::<_, u32>(2).slim_map_ok_async(plus_3);
        let future_2 = future.clone();

        assert_eq!(future.await, Ok(5));
        assert_eq!(future_2.await, Ok(5));
    }

    #[tokio::test]
    async fn test_map_ok_async_fused_future() {
        let mut future = future::ok::<_, u32>(2).slim_map_ok_async(plus_3);

        assert!(!future.is_terminated());
        assert_eq!(future.by_ref().await, Ok(5));
        assert!(future.is_terminated());
    }

    #[tokio::test]
    async fn test_map_ok_async_is_slim() {
        let make_base_future = || crate::future::ok_by_copy::<_, u32>(NonZeroU32::new(2).unwrap()).slim_map_ok(drop);
        let base_future = make_base_future();
        let future = make_base_future().slim_map_ok_async(crate::future::ready_by_copy);

        assert_eq!(mem::size_of_val(&base_future), mem::size_of_val(&future));
        assert_eq!(base_future.await, Ok(()));
        assert_eq!(future.await, Ok(()));
    }
}
