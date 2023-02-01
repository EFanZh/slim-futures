use crate::future::map::Map;
use crate::support::fns::MapErrFn;
use crate::support::ResultFuture;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::FnMut;
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    #[derive(Clone)]
    pub struct MapErr<Fut, F>
    where
        F: ?Sized,
    {
        #[pin]
        inner: Map<Fut, MapErrFn<F>>,
    }
}

impl<Fut, F> MapErr<Fut, F> {
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self {
            inner: Map::new(fut, MapErrFn::new(f)),
        }
    }
}

impl<Fut, F> Future for MapErr<Fut, F>
where
    Fut: ResultFuture,
    F: FnMut<(Fut::Error,)> + ?Sized,
{
    type Output = Result<Fut::Ok, F::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, F> FusedFuture for MapErr<Fut, F>
where
    Fut: ResultFuture + FusedFuture,
    F: FnMut<(Fut::Error,)> + ?Sized,
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
    async fn test_map_err() {
        assert_eq!(future::ok::<u32, _>(2).slim_map_err(plus_3).await, Ok(2));
        assert_eq!(future::err::<u32, _>(2).slim_map_err(plus_3).await, Err(5));
    }

    #[tokio::test]
    async fn test_map_err_clone() {
        let future = future::err::<u32, _>(2).slim_map_err(plus_3);
        let future_2 = future.clone();

        assert_eq!(future.await, Err(5));
        assert_eq!(future_2.await, Err(5));
    }

    #[tokio::test]
    async fn test_map_err_fused_future() {
        let mut future = future::err::<u32, _>(2).slim_map_err(plus_3);

        assert!(!future.is_terminated());
        assert_eq!(future.by_ref().await, Err(5));
        assert!(future.is_terminated());
    }

    #[tokio::test]
    async fn test_map_err_is_slim() {
        let make_base_future = || crate::future::err_by_copy::<u32, u32>(2);
        let base_future = make_base_future();
        let future_1 = make_base_future().slim_map_err(plus_3);
        let future_2 = make_base_future().map_err(plus_3);

        assert_eq!(mem::size_of_val(&base_future), mem::size_of_val(&future_1));
        assert!(mem::size_of_val(&future_1) < mem::size_of_val(&future_2));
        assert_eq!(base_future.await, Err(2));
        assert_eq!(future_1.await, Err(5));
        assert_eq!(future_2.await, Err(5));
    }
}
