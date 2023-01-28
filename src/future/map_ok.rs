use crate::future::map::Map;
use crate::support::fns::MapOkFn;
use crate::support::ResultFuture;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::FnMut;
use futures_core::FusedFuture;

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
            inner: Map::new(fut, MapOkFn::new(f)),
        }
    }
}

impl<Fut, F> Future for MapOk<Fut, F>
where
    Fut: ResultFuture,
    F: FnMut<(Fut::Ok,)>,
{
    type Output = Result<F::Output, Fut::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, F> FusedFuture for MapOk<Fut, F>
where
    Fut: ResultFuture + FusedFuture,
    F: FnMut<(Fut::Ok,)>,
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
        let make_base_future = || crate::future::ok_by_copy::<u32, u32>(2);
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
