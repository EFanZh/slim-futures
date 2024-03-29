use crate::future::and_then_async::AndThenAsync;
use crate::support::fns::MapOkAsyncFn;
use crate::support::{Residual, Try};
use core::future::{Future, IntoFuture};
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::FnMut;
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    pub struct MapOkAsync<Fut, F>
    where
        Fut: Future,
        Fut::Output: Try,
        <Fut::Output as Try>::Residual: Residual<<F::Output as IntoFuture>::Output>,
        F: FnMut<(<Fut::Output as Try>::Output,)>,
        F::Output: IntoFuture,
    {
        #[pin]
        inner: AndThenAsync<Fut, MapOkAsyncFn<F, <Fut::Output as Try>::Residual>>,
    }
}

impl<Fut, F> MapOkAsync<Fut, F>
where
    Fut: Future,
    Fut::Output: Try,
    <Fut::Output as Try>::Residual: Residual<<F::Output as IntoFuture>::Output>,
    F: FnMut<(<Fut::Output as Try>::Output,)>,
    F::Output: IntoFuture,
{
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self {
            inner: AndThenAsync::new(fut, MapOkAsyncFn::new(f)),
        }
    }
}

impl<Fut, F> Clone for MapOkAsync<Fut, F>
where
    Fut: Future + Clone,
    Fut::Output: Try,
    <Fut::Output as Try>::Residual: Residual<<F::Output as IntoFuture>::Output>,
    F: FnMut<(<Fut::Output as Try>::Output,)> + Clone,
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
    Fut: Future,
    Fut::Output: Try,
    <Fut::Output as Try>::Residual: Residual<<F::Output as IntoFuture>::Output>,
    F: FnMut<(<Fut::Output as Try>::Output,)>,
    F::Output: IntoFuture,
{
    type Output = <<Fut::Output as Try>::Residual as Residual<<F::Output as IntoFuture>::Output>>::TryType;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, F> FusedFuture for MapOkAsync<Fut, F>
where
    Fut: FusedFuture,
    Fut::Output: Try,
    <Fut::Output as Try>::Residual: Residual<<F::Output as IntoFuture>::Output>,
    F: FnMut<(<Fut::Output as Try>::Output,)>,
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
    use crate::future::{ok, ready};
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
    async fn test_map_ok_async_with_option() {
        assert_eq!(future::ready(Some(2)).slim_map_ok_async(plus_3).await, Some(5));
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
        let make_base_future = || ok::ok_by_copy::<_, u32>(NonZeroU32::new(2).unwrap()).slim_map_ok(drop);
        let base_future = make_base_future();
        let future = make_base_future().slim_map_ok_async(ready::ready_by_copy);

        assert_eq!(mem::size_of_val(&base_future), mem::size_of_val(&future));
        assert_eq!(base_future.await, Ok(()));
        assert_eq!(future.await, Ok(()));
    }
}
