use crate::future::map::Map;
use crate::support::fns::MapOkOrElseFn;
use crate::support::ResultFuture;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::FnMut;
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    #[derive(Clone)]
    pub struct MapOkOrElse<Fut, D, F> {
        #[pin]
        inner: Map<Fut, MapOkOrElseFn<D, F>>,
    }
}

impl<Fut, D, F> MapOkOrElse<Fut, D, F> {
    pub(crate) fn new(fut: Fut, default: D, f: F) -> Self {
        Self {
            inner: Map::new(fut, MapOkOrElseFn::new(default, f)),
        }
    }
}

impl<Fut, D, F> Future for MapOkOrElse<Fut, D, F>
where
    Fut: ResultFuture,
    D: FnMut<(Fut::Error,)>,
    F: FnMut<(Fut::Ok,), Output = D::Output>,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, D, F> FusedFuture for MapOkOrElse<Fut, D, F>
where
    Fut: ResultFuture + FusedFuture,
    D: FnMut<(Fut::Error,)>,
    F: FnMut<(Fut::Ok,), Output = D::Output>,
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

    fn plus_4(value: u32) -> u32 {
        value + 4
    }

    #[tokio::test]
    async fn test_map_ok_or_else() {
        assert_eq!(future::ok::<_, u32>(2).slim_map_ok_or_else(plus_3, plus_4).await, 6);
        assert_eq!(future::err::<_, u32>(2).slim_map_ok_or_else(plus_3, plus_4).await, 5);
    }

    #[tokio::test]
    async fn test_map_ok_or_else_clone() {
        let future = future::ok::<_, u32>(2).slim_map_ok_or_else(plus_3, plus_4);
        let future_2 = future.clone();

        assert_eq!(future.await, 6);
        assert_eq!(future_2.await, 6);
    }

    #[tokio::test]
    async fn test_map_ok_or_else_fused_future() {
        let mut future = future::ok::<_, u32>(2).slim_map_ok_or_else(plus_3, plus_4);

        assert!(!future.is_terminated());
        assert_eq!(future.by_ref().await, 6);
        assert!(future.is_terminated());
    }

    #[tokio::test]
    async fn test_map_ok_or_else_is_slim() {
        let make_base_future = || crate::future::ok_by_copy::<u32, u32>(2);
        let base_future = make_base_future();
        let future_1 = make_base_future().slim_map_ok_or_else(plus_3, plus_4);
        let future_2 = make_base_future().map_ok_or_else(plus_3, plus_4);

        assert_eq!(mem::size_of_val(&base_future), mem::size_of_val(&future_1));
        assert!(mem::size_of_val(&future_1) < mem::size_of_val(&future_2));
        assert_eq!(base_future.await, Ok(2));
        assert_eq!(future_1.await, 6);
        assert_eq!(future_2.await, 6);
    }
}
