use crate::future::flatten::Flatten;
use crate::future::map::Map;
use core::future::{Future, IntoFuture};
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::FnMut;
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    pub struct MapAsync<Fut, F>
    where
        Fut: Future,
        F: FnMut<(Fut::Output,)>,
        F::Output: IntoFuture,
    {
        #[pin]
        inner: Flatten<Map<Fut, F>>
    }
}

impl<Fut, F> MapAsync<Fut, F>
where
    Fut: Future,
    F: FnMut<(Fut::Output,)>,
    F::Output: IntoFuture,
{
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self {
            inner: Flatten::new(Map::new(fut, f)),
        }
    }
}

impl<Fut, F> Clone for MapAsync<Fut, F>
where
    Fut: Future + Clone,
    F: FnMut<(Fut::Output,)> + Clone,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::IntoFuture: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Fut, F> Future for MapAsync<Fut, F>
where
    Fut: Future,
    F: FnMut<(Fut::Output,)>,
    F::Output: IntoFuture,
{
    type Output = <F::Output as IntoFuture>::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, F> FusedFuture for MapAsync<Fut, F>
where
    Fut: FusedFuture,
    F: FnMut<(Fut::Output,)>,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::IntoFuture: FusedFuture,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::future;
    use crate::future::future_ext::FutureExt;
    use crate::future::ready;
    use futures_core::FusedFuture;
    use futures_util::FutureExt as _;
    use std::mem;
    use std::num::NonZeroU32;

    #[tokio::test]
    async fn test_map_async() {
        let future = ready::ready_by_copy(7).slim_map_async(|value| future::lazy(move |_| value + 2));

        assert_eq!(future.await, 9);
    }

    #[tokio::test]
    async fn test_map_async_clone() {
        let future = ready::ready_by_copy(7).slim_map_async(|value| future::lazy(move |_| value + 2));
        let future_2 = future.clone();

        assert_eq!(future.await, 9);
        assert_eq!(future_2.await, 9);
    }

    #[tokio::test]
    async fn test_map_async_fused_future() {
        let mut future =
            futures_util::future::ready(7).slim_map_async(|value| futures_util::future::lazy(move |_| value + 2));

        assert!(!future.is_terminated());
        assert_eq!(future.by_ref().await, 9);
        assert!(future.is_terminated());
    }

    #[tokio::test]
    async fn test_map_async_is_slim() {
        let make_base_future = || ready::ready_by_copy(NonZeroU32::new(2).unwrap()).slim_map(drop);
        let future_1 = make_base_future().slim_map_async(ready::ready_by_copy);
        let future_2 = make_base_future().then(ready::ready_by_copy);

        assert!(mem::size_of_val(&future_1) < mem::size_of_val(&future_2));
        assert!(matches!(future_1.await, ()));
        assert!(matches!(future_2.await, ()));
    }
}
