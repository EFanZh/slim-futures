use crate::future::map::Map;
use crate::support::fns::MapOkOrElseFn;
use crate::support::FnMut1;
use futures_core::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    #[derive(Clone)]
    pub struct MapOkOrElse<Fut, F, G> {
        #[pin]
        inner: Map<Fut, MapOkOrElseFn<F, G>>,
    }
}

impl<Fut, F, G> MapOkOrElse<Fut, F, G> {
    pub(crate) fn new(fut: Fut, ok_fn: F, err_fn: G) -> Self {
        Self {
            inner: Map::new(fut, MapOkOrElseFn::new(ok_fn, err_fn)),
        }
    }
}

impl<Fut, F, G, T, E> Future for MapOkOrElse<Fut, F, G>
where
    Fut: Future<Output = Result<T, E>>,
    F: FnMut1<T>,
    G: FnMut1<E, Output = F::Output>,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, F, G, T, E> FusedFuture for MapOkOrElse<Fut, F, G>
where
    Fut: FusedFuture<Output = Result<T, E>>,
    F: FnMut1<T>,
    G: FnMut1<E, Output = F::Output>,
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
        assert_eq!(future::ok::<_, u32>(2).slim_map_ok_or_else(plus_3, plus_4).await, 5);
        assert_eq!(future::err::<_, u32>(2).slim_map_ok_or_else(plus_3, plus_4).await, 6);
    }

    #[tokio::test]
    async fn test_map_ok_or_else_clone() {
        let future = future::ok::<_, u32>(2).slim_map_ok_or_else(plus_3, plus_4);
        let future_2 = future.clone();

        assert_eq!(future.await, 5);
        assert_eq!(future_2.await, 5);
    }

    #[tokio::test]
    async fn test_map_ok_or_else_fused_future() {
        let mut future = future::ok::<_, u32>(2).slim_map_ok_or_else(plus_3, plus_4);

        assert!(!future.is_terminated());
        assert_eq!((&mut future).await, 5);
        assert!(future.is_terminated());
    }

    #[tokio::test]
    async fn test_map_ok_or_else_is_slim() {
        let make_base_future = || crate::future::ok::<u32, u32>(2);
        let base_future = make_base_future();
        let future_1 = make_base_future().slim_map_ok_or_else(plus_3, plus_4);
        let future_2 = make_base_future().map_ok_or_else(plus_4, plus_3);

        assert_eq!(mem::size_of_val(&base_future), mem::size_of_val(&future_1));
        assert!(mem::size_of_val(&future_1) < mem::size_of_val(&future_2));
        assert_eq!(base_future.await, Ok(2));
        assert_eq!(future_1.await, 5);
        assert_eq!(future_2.await, 5);
    }
}
